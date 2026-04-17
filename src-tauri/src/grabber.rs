use crate::config::GestroConfig;
use crate::gesture::{GestureResult, GestureTracker};
use crate::simulator::{self, SIMULATING};
use crossbeam_channel::Receiver;
use rdev::{Button, Event, EventType};
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

            let grab_result = rdev::grab(move |event: Event| -> Option<Event> {
                // Check for config updates (non-blocking)
                if let Ok(new_config) = rx.try_recv() {
                    tracker.borrow_mut().update_config(&new_config);
                }

                // If we are replaying a right-click, pass through and count down
                if SIMULATING.load(Ordering::SeqCst) > 0 {
                    if matches!(
                        event.event_type,
                        EventType::ButtonPress(Button::Right)
                            | EventType::ButtonRelease(Button::Right)
                    ) {
                        SIMULATING.fetch_sub(1, Ordering::SeqCst);
                    }
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

            match grab_result {
                Ok(()) => {
                    log::info!("Grab loop exited cleanly");
                    return;
                }
                Err(e) => {
                    let hint = if cfg!(target_os = "linux") {
                        "Add your user to the 'input' group."
                    } else if cfg!(target_os = "macos") {
                        "Grant Accessibility permission in System Settings > Privacy & Security."
                    } else {
                        "Try running the app as administrator."
                    };
                    let msg = format!("Grab failed: {e:?}. {hint}");
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
