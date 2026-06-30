//! Small shared value types used across the engine API.

/// Stable handle for an entity registered with the engine (e.g. a collider).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u64);

/// Which edge of the world an entity crossed, reported by boundary detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}
