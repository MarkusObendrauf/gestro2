use crate::config::Shortcut;
use enigo::{Enigo, Keyboard, Settings};
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to prevent the grab callback from re-capturing synthetic events.
pub static SIMULATING: AtomicBool = AtomicBool::new(false);

/// Replay a right-click at the current cursor position.
/// Sets SIMULATING flag so the grab callback passes these events through.
pub fn replay_right_click() {
    SIMULATING.store(true, Ordering::SeqCst);

    let result = (|| {
        rdev::simulate(&rdev::EventType::ButtonPress(rdev::Button::Right))
            .map_err(|e| format!("Failed to simulate right press: {e:?}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        rdev::simulate(&rdev::EventType::ButtonRelease(rdev::Button::Right))
            .map_err(|e| format!("Failed to simulate right release: {e:?}"))?;
        Ok::<(), String>(())
    })();

    // Small delay to ensure events are processed before clearing flag
    std::thread::sleep(std::time::Duration::from_millis(20));
    SIMULATING.store(false, Ordering::SeqCst);

    if let Err(e) = result {
        log::error!("replay_right_click failed: {e}");
    }
}

/// Fire a keyboard shortcut using enigo.
pub fn fire_shortcut(shortcut: &Shortcut) {
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
        _ => None,
    }
}
