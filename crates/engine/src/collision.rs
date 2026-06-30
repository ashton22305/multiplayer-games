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
        let b = self.bounds;
        for c in &self.colliders {
            let bb = c.aabb();
            if bb.x < b.x { f(c.id, Side::Left); }
            if bb.x + bb.w > b.x + b.w { f(c.id, Side::Right); }
            if bb.y < b.y { f(c.id, Side::Top); }
            if bb.y + bb.h > b.y + b.h { f(c.id, Side::Bottom); }
        }
    }

    pub fn collisions(&self) -> Vec<(EntityId, EntityId)> {
        let mut out = Vec::new();
        self.for_each_collision(|a, b| out.push((a, b)));
        out
    }

    pub fn boundary_crossings(&self) -> Vec<(EntityId, Side)> {
        let mut out = Vec::new();
        self.for_each_boundary(|id, side| out.push((id, side)));
        out
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
        assert!(w.collisions().is_empty());
    }

    #[test]
    fn boundary_crossings_report_sides() {
        let mut w = CollisionWorld::new(Rect::new(0.0, 0.0, 100.0, 100.0));
        // pos (2, 50), width 10 → aabb.x = -3, crosses left boundary
        w.add(rect(0, 2.0, 50.0, 10.0, 10.0));
        let crossings = w.boundary_crossings();
        assert_eq!(crossings, vec![(EntityId(0), Side::Left)]);
    }

    #[test]
    fn callback_and_vec_apis_agree() {
        let mut w = CollisionWorld::new(Rect::new(0.0, 0.0, 100.0, 100.0));
        w.add(rect(0, 10.0, 10.0, 16.0, 16.0));
        w.add(rect(1, 15.0, 10.0, 16.0, 16.0));
        w.add(rect(2, 2.0, 50.0, 10.0, 10.0));

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
