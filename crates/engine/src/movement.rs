use crate::input::{direction_delta, Direction};
use macroquad::math::{vec2, IVec2, Vec2};

/// Tile-snapped actor for grid-based movement (Pac-Man, etc.).
///
/// Positions are in world units. The actor moves toward tile centers at a fixed
/// speed, snapping direction changes to tile boundaries so movement stays crisp.
pub struct TileActor {
    pub pos: Vec2,
    pub dir: Option<Direction>,
    pub want: Option<Direction>,
    pub speed: f32,
    cell_size: f32,
}

impl TileActor {
    pub fn new(tile: IVec2, cell_size: f32, speed: f32) -> Self {
        Self {
            pos: vec2(
                tile.x as f32 * cell_size + cell_size * 0.5,
                tile.y as f32 * cell_size + cell_size * 0.5,
            ),
            dir: None,
            want: None,
            speed,
            cell_size,
        }
    }

    pub fn tile(&self) -> IVec2 {
        IVec2::new(
            (self.pos.x / self.cell_size).floor() as i32,
            (self.pos.y / self.cell_size).floor() as i32,
        )
    }

    fn tile_center(&self, t: IVec2) -> Vec2 {
        vec2(
            t.x as f32 * self.cell_size + self.cell_size * 0.5,
            t.y as f32 * self.cell_size + self.cell_size * 0.5,
        )
    }

    /// Advance toward the next tile. `passable` returns true if the tile may be entered.
    /// Returns true on the frame the actor snaps to a tile center.
    pub fn advance(&mut self, dt: f32, passable: impl Fn(IVec2) -> bool) -> bool {
        let tile = self.tile();
        let center = self.tile_center(tile);
        let reached = (center - self.pos).length() <= self.speed * dt;
        if reached {
            self.pos = center;
            if let Some(w) = self.want {
                if passable(tile + direction_delta(w)) {
                    self.dir = Some(w);
                    self.want = None;
                }
            }
            if let Some(d) = self.dir {
                if !passable(tile + direction_delta(d)) {
                    self.dir = None;
                }
            }
        }
        if let Some(d) = self.dir {
            self.pos += direction_delta(d).as_vec2() * self.speed * dt;
        }
        reached
    }
}
