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

/// Fire a keyboard shortcut.
/// On macOS, uses CGEvent with modifier flags set directly on the key event to
/// avoid the race condition where macOS hasn't registered modifier state from a
/// separate press event. Falls back to enigo (on main thread) for media keys.
pub fn fire_shortcut(shortcut: &Shortcut) {
    #[cfg(target_os = "macos")]
    {
        // CGEvent path: bakes modifier flags into the key event itself — no race.
        if !shortcut.modifiers.is_empty() {
            if let Some(keycode) = string_to_macos_keycode(&shortcut.key) {
                fire_shortcut_cg(shortcut, keycode);
                return;
            }
        }
        // Enigo fallback for media keys or modifier-less shortcuts
        let shortcut = shortcut.clone();
        dispatch_to_main(move || fire_shortcut_impl(&shortcut));
    }
    #[cfg(not(target_os = "macos"))]
    fire_shortcut_impl(shortcut);
}

/// macOS: fire a shortcut using CGEvent with modifier flags embedded directly
/// in the key event. This is thread-safe (no dispatch_to_main needed) and
/// avoids the race condition inherent in separate modifier press + key press.
#[cfg(target_os = "macos")]
fn fire_shortcut_cg(shortcut: &Shortcut, keycode: u16) {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    // Build modifier flags to embed directly in the key event
    let mut flags = CGEventFlags::CGEventFlagNonCoalesced;
    for modifier in &shortcut.modifiers {
        match modifier.0.as_str() {
            "Meta" | "Super" | "Cmd" | "Command" => {
                flags |= CGEventFlags::CGEventFlagCommand;
            }
            "Ctrl" | "Control" => {
                flags |= CGEventFlags::CGEventFlagControl;
            }
            "Alt" => {
                flags |= CGEventFlags::CGEventFlagAlternate;
            }
            "Shift" => {
                flags |= CGEventFlags::CGEventFlagShift;
            }
            _ => log::warn!("Unknown modifier: {}", modifier.0),
        }
    }

    let post = |keydown: bool| -> Result<(), ()> {
        let src = CGEventSource::new(CGEventSourceStateID::HIDSystemState)?;
        let event = CGEvent::new_keyboard_event(src, keycode, keydown)?;
        event.set_flags(flags);
        event.post(CGEventTapLocation::HID);
        Ok(())
    };

    if post(true).is_err() || post(false).is_err() {
        log::error!("Failed to post CGEvent for key {}", shortcut.key);
        return;
    }

    log::info!(
        "Fired shortcut (CGEvent): {} + {}",
        shortcut
            .modifiers
            .iter()
            .map(|m| m.0.as_str())
            .collect::<Vec<_>>()
            .join("+"),
        shortcut.key
    );
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

    // Let the OS register modifier state before sending the main key.
    // Without this, macOS often misses the modifier (e.g. Meta+T → just T).
    if !shortcut.modifiers.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(20));
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
    // dispatch_get_main_queue() is a C macro expanding to &_dispatch_main_q
    extern "C" {
        static _dispatch_main_q: std::ffi::c_void;
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
        let queue = &_dispatch_main_q as *const std::ffi::c_void;
        dispatch_async_f(queue, context, trampoline::<F>);
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

/// Map key names to macOS virtual keycodes (Carbon HIToolbox/Events.h).
#[cfg(target_os = "macos")]
fn string_to_macos_keycode(name: &str) -> Option<u16> {
    if name.len() == 1 {
        let ch = name.chars().next().unwrap().to_ascii_lowercase();
        return match ch {
            'a' => Some(0x00), 's' => Some(0x01), 'd' => Some(0x02),
            'f' => Some(0x03), 'h' => Some(0x04), 'g' => Some(0x05),
            'z' => Some(0x06), 'x' => Some(0x07), 'c' => Some(0x08),
            'v' => Some(0x09), 'b' => Some(0x0B), 'q' => Some(0x0C),
            'w' => Some(0x0D), 'e' => Some(0x0E), 'r' => Some(0x0F),
            'y' => Some(0x10), 't' => Some(0x11), '1' => Some(0x12),
            '2' => Some(0x13), '3' => Some(0x14), '4' => Some(0x15),
            '6' => Some(0x16), '5' => Some(0x17), '=' => Some(0x18),
            '9' => Some(0x19), '7' => Some(0x1A), '-' => Some(0x1B),
            '8' => Some(0x1C), '0' => Some(0x1D), ']' => Some(0x1E),
            'o' => Some(0x1F), 'u' => Some(0x20), '[' => Some(0x21),
            'i' => Some(0x22), 'p' => Some(0x23), 'l' => Some(0x25),
            'j' => Some(0x26), '\'' => Some(0x27), 'k' => Some(0x28),
            ';' => Some(0x29), '\\' => Some(0x2A), ',' => Some(0x2B),
            '/' => Some(0x2C), 'n' => Some(0x2D), 'm' => Some(0x2E),
            '.' => Some(0x2F), '`' => Some(0x32),
            _ => None,
        };
    }
    match name {
        "Space" => Some(0x31),
        "Return" | "Enter" => Some(0x24),
        "Tab" => Some(0x30),
        "Escape" | "Esc" => Some(0x35),
        "Backspace" => Some(0x33),
        "Delete" => Some(0x75),
        "Up" => Some(0x7E),
        "Down" => Some(0x7D),
        "Left" => Some(0x7B),
        "Right" => Some(0x7C),
        "Home" => Some(0x73),
        "End" => Some(0x77),
        "PageUp" => Some(0x74),
        "PageDown" => Some(0x79),
        "F1" => Some(0x7A),
        "F2" => Some(0x78),
        "F3" => Some(0x63),
        "F4" => Some(0x76),
        "F5" => Some(0x60),
        "F6" => Some(0x61),
        "F7" => Some(0x62),
        "F8" => Some(0x64),
        "F9" => Some(0x65),
        "F10" => Some(0x6D),
        "F11" => Some(0x67),
        "F12" => Some(0x6F),
        _ => None,
    }
}
