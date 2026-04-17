use crate::config::GestroConfig;
use crate::gesture::{GestureResult, GestureTracker};
use crate::simulator::{self, SIMULATING};
use crossbeam_channel::Receiver;
use std::cell::RefCell;
use std::sync::atomic::Ordering;
use tauri::Emitter;

/// Max retry attempts before giving up.
const MAX_GRAB_RETRIES: u32 = 5;
/// Delay between retries (doubles each attempt).
const INITIAL_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

/// Spawn the grab thread. Receives config updates via the channel.
/// Retries on failure with exponential backoff.
pub fn spawn(app: tauri::AppHandle, config: GestroConfig, config_rx: Receiver<GestroConfig>) {
    std::thread::spawn(move || {
        let mut current_config = config;
        let mut delay = INITIAL_RETRY_DELAY;

        for attempt in 0..=MAX_GRAB_RETRIES {
            // Drain any config updates that arrived while we were retrying
            while let Ok(new_config) = config_rx.try_recv() {
                current_config = new_config;
            }

            let tracker = RefCell::new(GestureTracker::new(&current_config));
            let rx = config_rx.clone();

            if attempt > 0 {
                log::info!("Grab thread retry {attempt}/{MAX_GRAB_RETRIES}");
            } else {
                log::info!("Grab thread started");
            }

            let grab_result = run_grab(tracker, rx);

            match grab_result {
                Ok(()) => {
                    log::info!("Grab loop exited cleanly");
                    return;
                }
                Err(msg) => {
                    let hint = if cfg!(target_os = "linux") {
                        "Add your user to the 'input' group."
                    } else if cfg!(target_os = "macos") {
                        "Grant Accessibility permission in System Settings > Privacy & Security."
                    } else {
                        "Try running the app as administrator."
                    };
                    let msg = format!("{msg} {hint}");
                    log::error!("{msg}");
                    let _ = app.emit("grab-error", msg);

                    if attempt == MAX_GRAB_RETRIES {
                        log::error!("Grab thread giving up after {MAX_GRAB_RETRIES} retries");
                        return;
                    }

                    log::info!("Retrying grab in {}s...", delay.as_secs());
                    std::thread::sleep(delay);
                    delay *= 2;
                }
            }
        }
    });
}

/// Process a gesture event in the grab callback. Returns whether to suppress the event.
fn handle_event(
    tracker: &RefCell<GestureTracker>,
    rx: &Receiver<GestroConfig>,
    event_type: GrabEventType,
) -> bool {
    // Check for config updates (non-blocking)
    if let Ok(new_config) = rx.try_recv() {
        tracker.borrow_mut().update_config(&new_config);
    }

    // If we are replaying a right-click, pass through and count down
    if SIMULATING.load(Ordering::SeqCst) > 0 {
        if matches!(event_type, GrabEventType::RightPress | GrabEventType::RightRelease) {
            SIMULATING.fetch_sub(1, Ordering::SeqCst);
        }
        return false; // pass through
    }

    match event_type {
        GrabEventType::RightPress => {
            tracker.borrow_mut().start();
            true // suppress
        }
        GrabEventType::MouseMove { x, y } => {
            if tracker.borrow().is_tracking() {
                tracker.borrow_mut().update(x, y);
            }
            false // always pass through mouse moves
        }
        GrabEventType::RightRelease => {
            if !tracker.borrow().is_tracking() {
                return false; // pass through
            }

            let result = tracker.borrow_mut().finish();

            // Spawn the action on a separate thread to avoid blocking the grab callback
            std::thread::spawn(move || match result {
                GestureResult::PassThrough => {
                    log::debug!("Gesture pass-through: replaying right-click");
                    simulator::replay_right_click();
                }
                GestureResult::Fire(shortcut) => {
                    log::info!("Firing shortcut: {:?}", shortcut);
                    simulator::fire_shortcut(&shortcut);
                }
                GestureResult::Unbound(direction) => {
                    log::debug!("Unbound direction {direction}, replaying right-click");
                    simulator::replay_right_click();
                }
            });

            true // suppress
        }
        GrabEventType::Other => false,
    }
}

/// Simplified event type for our grab callback.
enum GrabEventType {
    RightPress,
    RightRelease,
    MouseMove { x: f64, y: f64 },
    Other,
}

// =============================================================================
// Linux / Windows: use rdev::grab
// =============================================================================
#[cfg(not(target_os = "macos"))]
fn run_grab(
    tracker: RefCell<GestureTracker>,
    rx: Receiver<GestroConfig>,
) -> Result<(), String> {
    use rdev::{Button, Event, EventType};

    let result = rdev::grab(move |event: Event| -> Option<Event> {
        let event_type = match event.event_type {
            EventType::ButtonPress(Button::Right) => GrabEventType::RightPress,
            EventType::ButtonRelease(Button::Right) => GrabEventType::RightRelease,
            EventType::MouseMove { x, y } => GrabEventType::MouseMove { x, y },
            _ => GrabEventType::Other,
        };

        let suppress = handle_event(&tracker, &rx, event_type);
        if suppress { None } else { Some(event) }
    });

    result.map_err(|e| format!("Grab failed: {e:?}."))
}

// =============================================================================
// macOS: use Core Graphics CGEventTap directly (mouse events only).
// rdev::grab crashes on macOS because it converts ALL events (including
// keyboard) via TSMGetInputSourceProperty, which must run on the main
// dispatch queue. By creating a mouse-only event tap, we avoid that path.
// =============================================================================
#[cfg(target_os = "macos")]
fn run_grab(
    tracker: RefCell<GestureTracker>,
    rx: Receiver<GestroConfig>,
) -> Result<(), String> {
    use core_graphics::event::*;
    use core_foundation::runloop::CFRunLoop;

    let events = vec![
        CGEventType::RightMouseDown,
        CGEventType::RightMouseUp,
        CGEventType::MouseMoved,
        CGEventType::RightMouseDragged,
    ];

    CGEventTap::with_enabled(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        events,
        |_proxy, etype, event| {
            let grab_event = match etype {
                CGEventType::RightMouseDown => GrabEventType::RightPress,
                CGEventType::RightMouseUp => GrabEventType::RightRelease,
                CGEventType::MouseMoved | CGEventType::RightMouseDragged => {
                    let loc = event.location();
                    GrabEventType::MouseMove { x: loc.x, y: loc.y }
                }
                _ => GrabEventType::Other,
            };

            if handle_event(&tracker, &rx, grab_event) {
                CallbackResult::Drop
            } else {
                CallbackResult::Keep
            }
        },
        || CFRunLoop::run_current(),
    )
    .map_err(|()| "Grab failed: CGEventTapCreate returned null (EventTapError).".to_string())?;

    Ok(())
}
