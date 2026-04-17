use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    /// All eight compass directions in clockwise order starting from North.
    pub const ALL: [Direction; 8] = [
        Direction::N,
        Direction::NE,
        Direction::E,
        Direction::SE,
        Direction::S,
        Direction::SW,
        Direction::W,
        Direction::NW,
    ];

    /// Classify a displacement vector (dx, dy) into one of 8 compass directions.
    /// Screen coordinates: +x is right, +y is down.
    /// Uses atan2 with negated dy so that "up" maps to North.
    pub fn classify(dx: f64, dy: f64) -> Direction {
        // atan2(-dy, dx) gives angle in radians where:
        //   0   = East
        //   π/2 = North (screen up)
        //   π   = West
        //  -π/2 = South (screen down)
        let angle = (-dy).atan2(dx);
        // Convert to degrees [0, 360)
        let deg = (angle.to_degrees() + 360.0) % 360.0;

        // Snap to nearest 45° sector
        // Each sector is 45° wide, centered on the cardinal/ordinal direction
        // Sector boundaries: 22.5, 67.5, 112.5, 157.5, 202.5, 247.5, 292.5, 337.5
        match deg {
            d if d < 22.5 => Direction::E,
            d if d < 67.5 => Direction::NE,
            d if d < 112.5 => Direction::N,
            d if d < 157.5 => Direction::NW,
            d if d < 202.5 => Direction::W,
            d if d < 247.5 => Direction::SW,
            d if d < 292.5 => Direction::S,
            d if d < 337.5 => Direction::SE,
            _ => Direction::E,
        }
    }

    /// Human-readable label for the direction.
    pub fn label(&self) -> &'static str {
        match self {
            Direction::N => "Up",
            Direction::NE => "Up-Right",
            Direction::E => "Right",
            Direction::SE => "Down-Right",
            Direction::S => "Down",
            Direction::SW => "Down-Left",
            Direction::W => "Left",
            Direction::NW => "Up-Left",
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinal_directions() {
        // Right (+x)
        assert_eq!(Direction::classify(100.0, 0.0), Direction::E);
        // Left (-x)
        assert_eq!(Direction::classify(-100.0, 0.0), Direction::W);
        // Up (screen: -y)
        assert_eq!(Direction::classify(0.0, -100.0), Direction::N);
        // Down (screen: +y)
        assert_eq!(Direction::classify(0.0, 100.0), Direction::S);
    }

    #[test]
    fn test_ordinal_directions() {
        // Up-right
        assert_eq!(Direction::classify(100.0, -100.0), Direction::NE);
        // Down-right
        assert_eq!(Direction::classify(100.0, 100.0), Direction::SE);
        // Down-left
        assert_eq!(Direction::classify(-100.0, 100.0), Direction::SW);
        // Up-left
        assert_eq!(Direction::classify(-100.0, -100.0), Direction::NW);
    }

    #[test]
    fn test_near_boundary() {
        // Just barely NE (slightly more up than right)
        assert_eq!(Direction::classify(51.0, -100.0), Direction::NE);
        // Just barely N (nearly straight up with tiny x)
        assert_eq!(Direction::classify(1.0, -100.0), Direction::N);
    }

    #[test]
    fn test_all_directions_present() {
        assert_eq!(Direction::ALL.len(), 8);
    }
}
