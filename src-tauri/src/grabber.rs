use crate::config::GestroConfig;
use crate::gesture::{GestureResult, GestureTracker};
use crate::simulator::{self, SIMULATING};
use crossbeam_channel::Receiver;
use rdev::{Button, Event, EventType};
use std::cell::RefCell;
use std::sync::atomic::Ordering;
use tauri::Emitter;

/// Spawn the grab thread. Receives config updates via the channel.
pub fn spawn(app: tauri::AppHandle, config: GestroConfig, config_rx: Receiver<GestroConfig>) {
    std::thread::spawn(move || {
        let tracker = RefCell::new(GestureTracker::new(&config));

        log::info!("Grab thread started");

        let grab_result = rdev::grab(move |event: Event| -> Option<Event> {
            // Check for config updates (non-blocking)
            if let Ok(new_config) = config_rx.try_recv() {
                tracker.borrow_mut().update_config(&new_config);
            }

            // If we are simulating events, pass through immediately
            if SIMULATING.load(Ordering::SeqCst) {
                return Some(event);
            }

            match event.event_type {
                EventType::ButtonPress(Button::Right) => {
                    // Suppress the right-click and start tracking
                    if let Some(name) = &event.name {
                        log::trace!("Suppressed right press: {name}");
                    }
                    // Use (0,0) as fallback — MouseMove events will update position
                    tracker.borrow_mut().start();
                    None // Suppress
                }
                EventType::MouseMove { x, y } => {
                    if tracker.borrow().is_tracking() {
                        // Update tracker position but let the move event through
                        // so the cursor keeps moving normally
                        tracker.borrow_mut().update(x, y);

                        // On first move after start, set the origin
                        // (since ButtonPress doesn't carry coordinates on all platforms)
                    }
                    Some(event) // Always pass through mouse moves
                }
                EventType::ButtonRelease(Button::Right) => {
                    if !tracker.borrow().is_tracking() {
                        return Some(event);
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

                    None // Suppress the release
                }
                _ => Some(event), // Pass through everything else
            }
        });

        if let Err(e) = grab_result {
            let msg = format!("Grab failed: {e:?}. Check that you have input permissions (e.g. add your user to the 'input' group on Linux).");
            log::error!("{msg}");
            let _ = app.emit("grab-error", msg);
        }
    });
}
