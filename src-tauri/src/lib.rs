mod config;
mod direction;
mod gesture;
mod grabber;
mod simulator;

use config::GestroConfig;
use crossbeam_channel::{unbounded, Sender};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_autostart::ManagerExt;

/// Shared state accessible from IPC commands.
struct AppState {
    config: Mutex<GestroConfig>,
    config_tx: Sender<GestroConfig>,
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> GestroConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn save_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    config: GestroConfig,
) -> Result<(), String> {
    config.save()?;

    // Toggle autostart if launch_at_login changed
    let old_launch = state.config.lock().unwrap().launch_at_login;
    if config.launch_at_login != old_launch {
        let autostart = app.autolaunch();
        if config.launch_at_login {
            autostart.enable().map_err(|e| format!("Failed to enable autostart: {e}"))?;
        } else {
            autostart.disable().map_err(|e| format!("Failed to disable autostart: {e}"))?;
        }
    }

    // Push updated config to the grab thread (non-fatal if grab thread is dead)
    if let Err(e) = state.config_tx.send(config.clone()) {
        log::warn!("Grab thread not running, config saved to disk only: {e}");
    }

    *state.config.lock().unwrap() = config;
    Ok(())
}

fn open_settings_window(app: &tauri::AppHandle) {
    // If the window already exists, focus it
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.set_focus();
        return;
    }

    // Create a new settings window on demand
    let _window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::default())
        .title("Gestro Settings")
        .inner_size(520.0, 600.0)
        .resizable(false)
        .center()
        .build()
        .expect("Failed to create settings window");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = GestroConfig::load();
    let (config_tx, config_rx) = unbounded();

    // Grab thread is spawned in setup() so we have an AppHandle
    let grab_config = config.clone();
    let grab_rx = config_rx;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState {
            config: Mutex::new(config),
            config_tx,
        })
        .invoke_handler(tauri::generate_handler![get_config, save_config])
        .setup(|app| {
            // On macOS, prompt for Accessibility permission before starting the grab thread.
            // AXIsProcessTrustedWithOptions with kAXTrustedCheckOptionPrompt shows the
            // system dialog directing the user to System Settings.
            #[cfg(target_os = "macos")]
            {
                extern "C" {
                    fn AXIsProcessTrustedWithOptions(
                        options: *const std::ffi::c_void,
                    ) -> bool;
                }
                extern "C" {
                    fn CFDictionaryCreate(
                        allocator: *const std::ffi::c_void,
                        keys: *const *const std::ffi::c_void,
                        values: *const *const std::ffi::c_void,
                        num_values: isize,
                        key_callbacks: *const std::ffi::c_void,
                        value_callbacks: *const std::ffi::c_void,
                    ) -> *const std::ffi::c_void;
                    fn CFRelease(cf: *const std::ffi::c_void);
                    static kCFBooleanTrue: *const std::ffi::c_void;
                    static kCFTypeDictionaryKeyCallBacks: u8;
                    static kCFTypeDictionaryValueCallBacks: u8;
                }
                // "AXTrustedCheckOptionPrompt" as a CFString
                let prompt_key: *const std::ffi::c_void = {
                    extern "C" {
                        fn CFStringCreateWithCString(
                            alloc: *const std::ffi::c_void,
                            c_str: *const u8,
                            encoding: u32,
                        ) -> *const std::ffi::c_void;
                    }
                    unsafe {
                        CFStringCreateWithCString(
                            std::ptr::null(),
                            b"AXTrustedCheckOptionPrompt\0".as_ptr(),
                            0x08000100, // kCFStringEncodingUTF8
                        )
                    }
                };
                let trusted = unsafe {
                    let keys = [prompt_key];
                    let values = [kCFBooleanTrue];
                    let dict = CFDictionaryCreate(
                        std::ptr::null(),
                        keys.as_ptr(),
                        values.as_ptr(),
                        1,
                        &kCFTypeDictionaryKeyCallBacks as *const u8 as *const std::ffi::c_void,
                        &kCFTypeDictionaryValueCallBacks as *const u8 as *const std::ffi::c_void,
                    );
                    let result = AXIsProcessTrustedWithOptions(dict);
                    CFRelease(dict);
                    CFRelease(prompt_key);
                    result
                };
                log::info!("macOS Accessibility trusted: {trusted}");
            }

            // Start the grab thread with access to app handle for error events
            grabber::spawn(app.handle().clone(), grab_config, grab_rx);

            // Build system tray
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => open_settings_window(app),
                    "quit" => {
                        log::info!("Quit requested from tray");
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Sync autostart desktop entry with current binary path on every launch.
            // This ensures the .desktop file stays correct if the binary moves.
            let autostart = app.handle().autolaunch();
            let launch_at_login = app.state::<AppState>().config.lock().unwrap().launch_at_login;
            if launch_at_login {
                if let Err(e) = autostart.enable() {
                    log::warn!("Failed to sync autostart: {e}");
                }
            }

            log::info!("Gestro started — listening for gestures");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Error building Tauri application")
        .run(|_app, event| {
            // Keep the app alive when all windows are closed,
            // but allow explicit quit (app.exit()) to go through.
            if let RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
