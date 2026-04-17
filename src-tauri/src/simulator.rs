use crate::config::Shortcut;
use enigo::{Enigo, Keyboard, Settings};
use std::sync::atomic::{AtomicU32, Ordering};

/// Count of simulated right-button events the grab callback should pass through.
/// Set before simulating; the grab callback decrements as it processes each one.
pub static SIMULATING: AtomicU32 = AtomicU32::new(0);

/// Replay a right-click at the current cursor position.
pub fn replay_right_click() {
    // Tell the grab callback to pass through the next 2 right-button events
    SIMULATING.store(2, Ordering::SeqCst);

    let press_ok = rdev::simulate(&rdev::EventType::ButtonPress(rdev::Button::Right));
    std::thread::sleep(std::time::Duration::from_millis(10));
    let release_ok = rdev::simulate(&rdev::EventType::ButtonRelease(rdev::Button::Right));

    if let Err(ref e) = press_ok {
        log::error!("Failed to simulate right press: {e:?}");
    }
    if let Err(ref e) = release_ok {
        log::error!("Failed to simulate right release: {e:?}");
    }

    if press_ok.is_err() || release_ok.is_err() {
        SIMULATING.store(0, Ordering::SeqCst);
        return;
    }

    // Wait for the grab callback to process both events (up to 200ms)
    for _ in 0..20 {
        if SIMULATING.load(Ordering::SeqCst) == 0 {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Safety: force-clear if events weren't processed in time
    let remaining = SIMULATING.swap(0, Ordering::SeqCst);
    if remaining > 0 {
        log::warn!("Simulated right-click events not processed in time ({remaining} remaining), force-cleared");
    }
}

/// Fire a keyboard shortcut using enigo.
/// On macOS, enigo's key lookup calls TSMGetInputSourceProperty which must run
/// on the main thread. We dispatch via GCD to avoid crashing.
pub fn fire_shortcut(shortcut: &Shortcut) {
    #[cfg(target_os = "macos")]
    {
        let shortcut = shortcut.clone();
        dispatch_to_main(move || fire_shortcut_impl(&shortcut));
    }
    #[cfg(not(target_os = "macos"))]
    fire_shortcut_impl(shortcut);
}

fn fire_shortcut_impl(shortcut: &Shortcut) {
    let Ok(mut enigo) = Enigo::new(&Settings::default()) else {
        log::error!("Failed to create Enigo instance");
        return;
    };

    // Press modifiers
    for modifier in &shortcut.modifiers {
        if let Some(key) = modifier_to_enigo(&modifier.0) {
            if let Err(e) = enigo.key(key, enigo::Direction::Press) {
                log::error!("Failed to press modifier {}: {e:?}", modifier.0);
                return;
            }
        }
    }

    // Press and release the main key
    if let Some(key) = string_to_enigo_key(&shortcut.key) {
        if let Err(e) = enigo.key(key, enigo::Direction::Click) {
            log::error!("Failed to click key {}: {e:?}", shortcut.key);
        }
    } else {
        log::error!("Unknown key: {}", shortcut.key);
    }

    // Release modifiers in reverse order
    for modifier in shortcut.modifiers.iter().rev() {
        if let Some(key) = modifier_to_enigo(&modifier.0) {
            if let Err(e) = enigo.key(key, enigo::Direction::Release) {
                log::error!("Failed to release modifier {}: {e:?}", modifier.0);
            }
        }
    }

    log::info!(
        "Fired shortcut: {} + {}",
        shortcut
            .modifiers
            .iter()
            .map(|m| m.0.as_str())
            .collect::<Vec<_>>()
            .join("+"),
        shortcut.key
    );
}

/// Dispatch a closure to the main GCD queue (macOS only).
/// enigo's keyboard simulation calls TSMGetInputSourceProperty which
/// requires the main dispatch queue.
#[cfg(target_os = "macos")]
fn dispatch_to_main<F: FnOnce() + Send + 'static>(f: F) {
    extern "C" {
        fn dispatch_get_main_queue() -> *const std::ffi::c_void;
        fn dispatch_async_f(
            queue: *const std::ffi::c_void,
            context: *mut std::ffi::c_void,
            work: unsafe extern "C" fn(*mut std::ffi::c_void),
        );
    }

    unsafe extern "C" fn trampoline<F: FnOnce()>(context: *mut std::ffi::c_void) {
        let f = unsafe { Box::from_raw(context as *mut F) };
        f();
    }

    let context = Box::into_raw(Box::new(f)) as *mut std::ffi::c_void;
    unsafe {
        dispatch_async_f(dispatch_get_main_queue(), context, trampoline::<F>);
    }
}

fn modifier_to_enigo(name: &str) -> Option<enigo::Key> {
    match name {
        "Ctrl" | "Control" => Some(enigo::Key::Control),
        "Alt" => Some(enigo::Key::Alt),
        "Shift" => Some(enigo::Key::Shift),
        "Meta" | "Super" | "Cmd" | "Command" => Some(enigo::Key::Meta),
        _ => {
            log::warn!("Unknown modifier: {name}");
            None
        }
    }
}

fn string_to_enigo_key(name: &str) -> Option<enigo::Key> {
    // Single character keys
    if name.len() == 1 {
        let ch = name.chars().next().unwrap();
        return Some(enigo::Key::Unicode(ch.to_ascii_lowercase()));
    }

    match name {
        "Space" => Some(enigo::Key::Space),
        "Return" | "Enter" => Some(enigo::Key::Return),
        "Tab" => Some(enigo::Key::Tab),
        "Escape" | "Esc" => Some(enigo::Key::Escape),
        "Backspace" => Some(enigo::Key::Backspace),
        "Delete" => Some(enigo::Key::Delete),
        "Up" => Some(enigo::Key::UpArrow),
        "Down" => Some(enigo::Key::DownArrow),
        "Left" => Some(enigo::Key::LeftArrow),
        "Right" => Some(enigo::Key::RightArrow),
        "Home" => Some(enigo::Key::Home),
        "End" => Some(enigo::Key::End),
        "PageUp" => Some(enigo::Key::PageUp),
        "PageDown" => Some(enigo::Key::PageDown),
        "F1" => Some(enigo::Key::F1),
        "F2" => Some(enigo::Key::F2),
        "F3" => Some(enigo::Key::F3),
        "F4" => Some(enigo::Key::F4),
        "F5" => Some(enigo::Key::F5),
        "F6" => Some(enigo::Key::F6),
        "F7" => Some(enigo::Key::F7),
        "F8" => Some(enigo::Key::F8),
        "F9" => Some(enigo::Key::F9),
        "F10" => Some(enigo::Key::F10),
        "F11" => Some(enigo::Key::F11),
        "F12" => Some(enigo::Key::F12),
        // Media keys (cross-platform enigo variants)
        "MediaPlayPause" => Some(enigo::Key::MediaPlayPause),
        "MediaNextTrack" | "MediaTrackNext" => Some(enigo::Key::MediaNextTrack),
        "MediaPrevTrack" | "MediaTrackPrevious" => Some(enigo::Key::MediaPrevTrack),
        "AudioVolumeUp" | "VolumeUp" => Some(enigo::Key::VolumeUp),
        "AudioVolumeDown" | "VolumeDown" => Some(enigo::Key::VolumeDown),
        "AudioVolumeMute" | "VolumeMute" => Some(enigo::Key::VolumeMute),
        // Browser keys — platform-specific mappings
        "BrowserBack" => browser_key_back(),
        "BrowserForward" => browser_key_forward(),
        "BrowserRefresh" => browser_key_refresh(),
        "BrowserHome" => browser_key_home(),
        _ => None,
    }
}

// Browser keys: enigo has named variants on Windows, raw X11 keysyms on Linux.
// macOS has no standard keycodes for browser navigation — users should bind
// Cmd+[ / Cmd+] etc. instead.

#[cfg(target_os = "windows")]
fn browser_key_back() -> Option<enigo::Key> { Some(enigo::Key::BrowserBack) }
#[cfg(target_os = "windows")]
fn browser_key_forward() -> Option<enigo::Key> { Some(enigo::Key::BrowserForward) }
#[cfg(target_os = "windows")]
fn browser_key_refresh() -> Option<enigo::Key> { Some(enigo::Key::BrowserRefresh) }
#[cfg(target_os = "windows")]
fn browser_key_home() -> Option<enigo::Key> { Some(enigo::Key::BrowserHome) }

#[cfg(all(unix, not(target_os = "macos")))]
fn browser_key_back() -> Option<enigo::Key> { Some(enigo::Key::Other(0x1008FF26)) }
#[cfg(all(unix, not(target_os = "macos")))]
fn browser_key_forward() -> Option<enigo::Key> { Some(enigo::Key::Other(0x1008FF27)) }
#[cfg(all(unix, not(target_os = "macos")))]
fn browser_key_refresh() -> Option<enigo::Key> { Some(enigo::Key::Other(0x1008FF29)) }
#[cfg(all(unix, not(target_os = "macos")))]
fn browser_key_home() -> Option<enigo::Key> { Some(enigo::Key::Other(0x1008FF18)) }

#[cfg(target_os = "macos")]
fn browser_key_back() -> Option<enigo::Key> { None }
#[cfg(target_os = "macos")]
fn browser_key_forward() -> Option<enigo::Key> { None }
#[cfg(target_os = "macos")]
fn browser_key_refresh() -> Option<enigo::Key> { None }
#[cfg(target_os = "macos")]
fn browser_key_home() -> Option<enigo::Key> { None }
