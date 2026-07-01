use crate::types::EntityId;
use macroquad::math::{Rect, Vec2};

/// Which edge of the world an entity crossed, reported by boundary detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub id: EntityId,
    pub pos: Vec2,
    pub width: f32,
    pub height: f32,
    pub layer: u32,
    pub mask: u32,
}

impl Collider {
    pub fn new(id: EntityId, pos: Vec2, width: f32, height: f32) -> Self {
        Self { id, pos, width, height, layer: 1, mask: u32::MAX }
    }

    pub fn with_layers(mut self, layer: u32, mask: u32) -> Self {
        self.layer = layer;
        self.mask = mask;
        self
    }

    pub fn aabb(&self) -> Rect {
        Rect::new(
            self.pos.x - self.width * 0.5,
            self.pos.y - self.height * 0.5,
            self.width,
            self.height,
        )
    }
}

fn layers_match(a: &Collider, b: &Collider) -> bool {
    (a.mask & b.layer) != 0 || (b.mask & a.layer) != 0
}

/// Which edges of `bounds` the given box extends past, if any.
fn crossed_sides(bounds: Rect, aabb: Rect) -> impl Iterator<Item = Side> {
    [
        (aabb.x < bounds.x, Side::Left),
        (aabb.x + aabb.w > bounds.x + bounds.w, Side::Right),
        (aabb.y < bounds.y, Side::Top),
        (aabb.y + aabb.h > bounds.y + bounds.h, Side::Bottom),
    ]
    .into_iter()
    .filter_map(|(crossed, side)| crossed.then_some(side))
}

/// Whether a box extends past any edge of `bounds`. Usable outside a
/// `CollisionWorld` for games that need an immediate, synchronous answer
/// (e.g. a grid game deciding mid-tick whether a move is still in bounds).
pub fn out_of_bounds(bounds: Rect, aabb: Rect) -> bool {
    crossed_sides(bounds, aabb).next().is_some()
}

pub struct CollisionWorld {
    bounds: Rect,
    colliders: Vec<Collider>,
}

impl CollisionWorld {
    pub fn new(bounds: Rect) -> Self {
        Self { bounds, colliders: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.colliders.clear();
    }

    pub fn add(&mut self, collider: Collider) {
        self.colliders.push(collider);
    }

    pub fn for_each_collision(&self, mut f: impl FnMut(EntityId, EntityId)) {
        for i in 0..self.colliders.len() {
            for j in (i + 1)..self.colliders.len() {
                let (a, b) = (&self.colliders[i], &self.colliders[j]);
                if layers_match(a, b) && a.aabb().overlaps(&b.aabb()) {
                    f(a.id, b.id);
                }
            }
        }
    }

    pub fn for_each_boundary(&self, mut f: impl FnMut(EntityId, Side)) {
        for c in &self.colliders {
            for side in crossed_sides(self.bounds, c.aabb()) {
                f(c.id, side);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::math::vec2;

    fn rect(id: u64, x: f32, y: f32, w: f32, h: f32) -> Collider {
        Collider::new(EntityId(id), vec2(x, y), w, h)
    }

    #[test]
    fn rects_overlap_on_aabb() {
        let a = rect(0, 0.0, 0.0, 20.0, 20.0);
        let b = rect(1, 10.0, 10.0, 20.0, 20.0);
        let c = rect(2, 40.0, 0.0, 20.0, 20.0);
        assert!(a.aabb().overlaps(&b.aabb()));
        assert!(!a.aabb().overlaps(&c.aabb()));
    }

    #[test]
    fn layer_masks_filter_pairs() {
        let mut w = CollisionWorld::new(Rect::new(-100.0, -100.0, 200.0, 200.0));
        w.add(rect(0, 0.0, 0.0, 20.0, 20.0).with_layers(0b01, 0b10));
        w.add(rect(1, 5.0, 0.0, 20.0, 20.0).with_layers(0b01, 0b10));
        let mut collisions = Vec::new();
        w.for_each_collision(|a, b| collisions.push((a, b)));
        assert!(collisions.is_empty());
    }

    #[test]
    fn boundary_crossings_report_sides() {
        let mut w = CollisionWorld::new(Rect::new(0.0, 0.0, 100.0, 100.0));
        // pos (2, 50), width 10 → aabb.x = -3, crosses left boundary
        w.add(rect(0, 2.0, 50.0, 10.0, 10.0));
        let mut crossings = Vec::new();
        w.for_each_boundary(|id, side| crossings.push((id, side)));
        assert_eq!(crossings, vec![(EntityId(0), Side::Left)]);
    }
}
