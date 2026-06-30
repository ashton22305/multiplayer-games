//! Lightweight collision and boundary detection. Games register colliders each
//! frame via [`Context`](crate::Context); the runtime then reports overlapping
//! pairs and boundary crossings through [`Game::on_collision`](crate::Game::on_collision)
//! and [`Game::on_boundary`](crate::Game::on_boundary).
//!
//! Broadphase is naive O(n^2), which is ample for the games this engine targets;
//! it can be swapped for a spatial grid later without changing the API.

use crate::types::{EntityId, Side};
use macroquad::math::{vec2, Rect, Vec2};

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    /// Axis-aligned box centered on the collider position.
    Rect { width: f32, height: f32 },
    /// Circle centered on the collider position.
    Circle { radius: f32 },
}

#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub id: EntityId,
    pub pos: Vec2,
    pub shape: Shape,
    /// Bitset of layers this collider belongs to.
    pub layer: u32,
    /// Bitset of layers this collider tests against.
    pub mask: u32,
}

impl Collider {
    /// A collider on layer 1 that tests against everything.
    pub fn new(id: EntityId, pos: Vec2, shape: Shape) -> Self {
        Self {
            id,
            pos,
            shape,
            layer: 1,
            mask: u32::MAX,
        }
    }

    pub fn with_layers(mut self, layer: u32, mask: u32) -> Self {
        self.layer = layer;
        self.mask = mask;
        self
    }

    fn half_extents(&self) -> Vec2 {
        match self.shape {
            Shape::Rect { width, height } => vec2(width * 0.5, height * 0.5),
            Shape::Circle { radius } => vec2(radius, radius),
        }
    }

    /// Bounding box in world space.
    pub fn aabb(&self) -> Rect {
        let h = self.half_extents();
        Rect::new(self.pos.x - h.x, self.pos.y - h.y, h.x * 2.0, h.y * 2.0)
    }
}

fn circle_rect(center: Vec2, radius: f32, rect: Rect) -> bool {
    let closest = vec2(
        center.x.clamp(rect.x, rect.x + rect.w),
        center.y.clamp(rect.y, rect.y + rect.h),
    );
    center.distance_squared(closest) <= radius * radius
}

fn overlaps(a: &Collider, b: &Collider) -> bool {
    match (a.shape, b.shape) {
        (Shape::Circle { radius: ra }, Shape::Circle { radius: rb }) => {
            a.pos.distance_squared(b.pos) <= (ra + rb) * (ra + rb)
        }
        (Shape::Rect { .. }, Shape::Rect { .. }) => a.aabb().overlaps(&b.aabb()),
        (Shape::Circle { radius }, Shape::Rect { .. }) => circle_rect(a.pos, radius, b.aabb()),
        (Shape::Rect { .. }, Shape::Circle { radius }) => circle_rect(b.pos, radius, a.aabb()),
    }
}

/// Two colliders interact if either one's mask includes the other's layer.
fn layers_match(a: &Collider, b: &Collider) -> bool {
    (a.mask & b.layer) != 0 || (b.mask & a.layer) != 0
}

/// Per-frame set of colliders. Cleared and repopulated by the game each frame.
pub struct CollisionWorld {
    bounds: Rect,
    colliders: Vec<Collider>,
}

impl CollisionWorld {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            colliders: Vec::new(),
        }
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    pub fn clear(&mut self) {
        self.colliders.clear();
    }

    pub fn add(&mut self, collider: Collider) {
        self.colliders.push(collider);
    }

    /// Call `f` for each overlapping, layer-compatible pair (reported once each).
    /// Prefer this over [`collisions`] in the render loop to avoid per-frame allocation.
    pub fn for_each_collision(&self, mut f: impl FnMut(EntityId, EntityId)) {
        for i in 0..self.colliders.len() {
            for j in (i + 1)..self.colliders.len() {
                let (a, b) = (&self.colliders[i], &self.colliders[j]);
                if layers_match(a, b) && overlaps(a, b) {
                    f(a.id, b.id);
                }
            }
        }
    }

    /// Call `f` for each collider whose bounding box extends past the world.
    /// A corner crossing yields two calls for the same entity.
    /// Prefer this over [`boundary_crossings`] in the render loop to avoid per-frame allocation.
    pub fn for_each_boundary(&self, mut f: impl FnMut(EntityId, Side)) {
        let b = self.bounds;
        for c in &self.colliders {
            let bb = c.aabb();
            if bb.x < b.x {
                f(c.id, Side::Left);
            }
            if bb.x + bb.w > b.x + b.w {
                f(c.id, Side::Right);
            }
            if bb.y < b.y {
                f(c.id, Side::Top);
            }
            if bb.y + bb.h > b.y + b.h {
                f(c.id, Side::Bottom);
            }
        }
    }

    /// Each overlapping, layer-compatible pair, reported once.
    /// Allocates a `Vec`; use [`for_each_collision`](Self::for_each_collision) in hot paths.
    pub fn collisions(&self) -> Vec<(EntityId, EntityId)> {
        let mut out = Vec::new();
        self.for_each_collision(|a, b| out.push((a, b)));
        out
    }

    /// Colliders whose bounding box extends past the world, with the side(s) crossed.
    /// Allocates a `Vec`; use [`for_each_boundary`](Self::for_each_boundary) in hot paths.
    pub fn boundary_crossings(&self) -> Vec<(EntityId, Side)> {
        let mut out = Vec::new();
        self.for_each_boundary(|id, side| out.push((id, side)));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(id: u64, x: f32, y: f32, r: f32) -> Collider {
        Collider::new(EntityId(id), vec2(x, y), Shape::Circle { radius: r })
    }
    fn rect(id: u64, x: f32, y: f32, w: f32, h: f32) -> Collider {
        Collider::new(
            EntityId(id),
            vec2(x, y),
            Shape::Rect {
                width: w,
                height: h,
            },
        )
    }

    #[test]
    fn circles_overlap_when_close() {
        assert!(overlaps(
            &circle(0, 0.0, 0.0, 10.0),
            &circle(1, 15.0, 0.0, 10.0)
        ));
        assert!(!overlaps(
            &circle(0, 0.0, 0.0, 10.0),
            &circle(1, 25.0, 0.0, 10.0)
        ));
    }

    #[test]
    fn rects_overlap_on_aabb() {
        assert!(overlaps(
            &rect(0, 0.0, 0.0, 20.0, 20.0),
            &rect(1, 10.0, 10.0, 20.0, 20.0)
        ));
        assert!(!overlaps(
            &rect(0, 0.0, 0.0, 20.0, 20.0),
            &rect(1, 40.0, 0.0, 20.0, 20.0)
        ));
    }

    #[test]
    fn circle_against_rect() {
        // Circle just touching the right edge of a box centered at origin.
        assert!(overlaps(
            &rect(0, 0.0, 0.0, 20.0, 20.0),
            &circle(1, 14.0, 0.0, 5.0)
        ));
        assert!(!overlaps(
            &rect(0, 0.0, 0.0, 20.0, 20.0),
            &circle(1, 20.0, 0.0, 5.0)
        ));
    }

    #[test]
    fn layer_masks_filter_pairs() {
        let a = circle(0, 0.0, 0.0, 10.0).with_layers(0b01, 0b10);
        let b = circle(1, 5.0, 0.0, 10.0).with_layers(0b01, 0b10); // same layer, masks don't include it
        let mut w = CollisionWorld::new(Rect::new(-100.0, -100.0, 200.0, 200.0));
        w.add(a);
        w.add(b);
        assert!(w.collisions().is_empty());
    }

    #[test]
    fn boundary_crossings_report_sides() {
        let mut w = CollisionWorld::new(Rect::new(0.0, 0.0, 100.0, 100.0));
        w.add(circle(0, 2.0, 50.0, 5.0)); // pokes past the left edge
        let crossings = w.boundary_crossings();
        assert_eq!(crossings, vec![(EntityId(0), Side::Left)]);
    }

    #[test]
    fn callback_and_vec_apis_agree() {
        let mut w = CollisionWorld::new(Rect::new(0.0, 0.0, 100.0, 100.0));
        w.add(circle(0, 10.0, 10.0, 8.0));
        w.add(circle(1, 15.0, 10.0, 8.0)); // overlaps 0
        w.add(circle(2, 2.0, 50.0, 5.0)); // crosses left boundary

        let vec_collisions = w.collisions();
        let mut cb_collisions = Vec::new();
        w.for_each_collision(|a, b| cb_collisions.push((a, b)));
        assert_eq!(vec_collisions, cb_collisions);

        let vec_crossings = w.boundary_crossings();
        let mut cb_crossings = Vec::new();
        w.for_each_boundary(|id, side| cb_crossings.push((id, side)));
        assert_eq!(vec_crossings, cb_crossings);
    }
}
