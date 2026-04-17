use crate::config::{GestroConfig, Shortcut};
use crate::direction::Direction;

/// Result of completing a gesture.
#[derive(Debug)]
pub enum GestureResult {
    /// Displacement below threshold — replay the right-click.
    PassThrough,
    /// Direction matched a binding — fire this shortcut.
    Fire(Shortcut),
    /// Direction matched but no binding assigned — replay the right-click.
    Unbound(Direction),
}

/// Tracks mouse movement during a right-click hold to detect directional gestures.
pub struct GestureTracker {
    /// Whether we are actively tracking a gesture.
    tracking: bool,
    /// Starting position of the gesture (set on first mouse move after start).
    origin: Option<(f64, f64)>,
    /// Current position during tracking.
    current: (f64, f64),
    /// Pixel displacement threshold before a gesture is recognized.
    threshold: f64,
    /// Direction-to-shortcut bindings.
    bindings: std::collections::HashMap<Direction, Shortcut>,
}

impl GestureTracker {
    pub fn new(config: &GestroConfig) -> Self {
        Self {
            tracking: false,
            origin: None,
            current: (0.0, 0.0),
            threshold: config.threshold,
            bindings: config.bindings.clone(),
        }
    }

    /// Begin tracking a gesture. Origin is captured on the first mouse move.
    pub fn start(&mut self) {
        self.tracking = true;
        self.origin = None;
        log::debug!("Gesture tracking started");
    }

    /// Update the current mouse position during tracking.
    /// Sets origin on the first call after start.
    pub fn update(&mut self, x: f64, y: f64) {
        if self.origin.is_none() {
            self.origin = Some((x, y));
            log::debug!("Gesture origin set at ({x}, {y})");
        }
        self.current = (x, y);
    }

    /// Returns true if we are currently tracking a gesture.
    pub fn is_tracking(&self) -> bool {
        self.tracking
    }

    /// Finish the gesture and determine the result.
    pub fn finish(&mut self) -> GestureResult {
        self.tracking = false;
        let Some((ox, oy)) = self.origin.take() else {
            return GestureResult::PassThrough;
        };

        let (cx, cy) = self.current;
        let dx = cx - ox;
        let dy = cy - oy;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < self.threshold {
            log::debug!("Gesture below threshold ({distance:.1} < {})", self.threshold);
            return GestureResult::PassThrough;
        }

        let direction = Direction::classify(dx, dy);
        log::info!("Gesture detected: {direction} (distance: {distance:.1}px)");

        match self.bindings.get(&direction) {
            Some(shortcut) => GestureResult::Fire(shortcut.clone()),
            None => GestureResult::Unbound(direction),
        }
    }

    /// Update config (called when settings change at runtime).
    pub fn update_config(&mut self, config: &GestroConfig) {
        self.threshold = config.threshold;
        self.bindings = config.bindings.clone();
        log::info!("GestureTracker config updated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GestroConfig, Modifier, Shortcut};
    use crate::direction::Direction;
    use std::collections::HashMap;

    fn test_config() -> GestroConfig {
        let mut bindings = HashMap::new();
        bindings.insert(
            Direction::E,
            Shortcut {
                modifiers: vec![Modifier("Ctrl".into())],
                key: "Right".into(),
                label: Some("Test East".into()),
            },
        );
        GestroConfig {
            threshold: 50.0,
            bindings,
            launch_at_login: false,
        }
    }

    #[test]
    fn test_below_threshold_returns_passthrough() {
        let config = test_config();
        let mut tracker = GestureTracker::new(&config);
        tracker.start();
        tracker.update(100.0, 100.0); // origin set here
        tracker.update(110.0, 105.0); // only ~11px movement
        match tracker.finish() {
            GestureResult::PassThrough => {}
            other => panic!("Expected PassThrough, got {other:?}"),
        }
    }

    #[test]
    fn test_bound_direction_fires() {
        let config = test_config();
        let mut tracker = GestureTracker::new(&config);
        tracker.start();
        tracker.update(100.0, 100.0); // origin set here
        tracker.update(200.0, 100.0); // 100px East
        match tracker.finish() {
            GestureResult::Fire(shortcut) => {
                assert_eq!(shortcut.key, "Right");
            }
            other => panic!("Expected Fire, got {other:?}"),
        }
    }

    #[test]
    fn test_unbound_direction() {
        let config = test_config();
        let mut tracker = GestureTracker::new(&config);
        tracker.start();
        tracker.update(100.0, 100.0); // origin set here
        tracker.update(100.0, 0.0); // 100px North (no binding)
        match tracker.finish() {
            GestureResult::Unbound(dir) => assert_eq!(dir, Direction::N),
            other => panic!("Expected Unbound, got {other:?}"),
        }
    }

    #[test]
    fn test_finish_without_start_returns_passthrough() {
        let config = test_config();
        let mut tracker = GestureTracker::new(&config);
        match tracker.finish() {
            GestureResult::PassThrough => {}
            other => panic!("Expected PassThrough, got {other:?}"),
        }
    }
}
