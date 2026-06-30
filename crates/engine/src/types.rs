/// Stable handle for an entity registered with the engine (e.g. a collider).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u64);
