use engine::macroquad::prelude::*;
use engine::macroquad::rand;
use engine::protocol::HostEvent;
use engine::{
    direction_delta, host, Action, Collider, Context, Direction, EntityId, Game, GameConfig, Gfx,
    TileActor,
};

const COLS: i32 = 19;
const ROWS: i32 = 21;
const CELL: f32 = 32.0;
const WORLD_W: f32 = COLS as f32 * CELL;
const WORLD_H: f32 = ROWS as f32 * CELL;

const PAC_SPEED: f32 = 110.0;
const GHOST_SPEED: f32 = 95.0;
const FRIGHT_TIME: f32 = 6.0;

const PELLET_SCORE: u32 = 1;
const POWER_SCORE: u32 = 5;
const GHOST_SCORE: u32 = 20;

const PAC_ID: EntityId = EntityId(0);
const LAYER_PAC: u32 = 0b01;
const LAYER_GHOST: u32 = 0b10;

const PAC_START: IVec2 = IVec2::new(9, 15);
const GHOST_STARTS: [IVec2; 3] = [IVec2::new(7, 9), IVec2::new(9, 9), IVec2::new(11, 9)];

const ALL_DIRS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

fn is_wall(t: IVec2) -> bool {
    let (x, y) = (t.x, t.y);
    x <= 0 || y <= 0 || x >= COLS - 1 || y >= ROWS - 1 || (x % 2 == 0 && y % 2 == 0)
}

fn tile_center(t: IVec2) -> Vec2 {
    vec2(t.x as f32 * CELL + CELL * 0.5, t.y as f32 * CELL + CELL * 0.5)
}

enum Phase {
    Playing,
    Lost,
    Won,
}

struct Pacman {
    pac: TileActor,
    ghosts: Vec<TileActor>,
    cells: Vec<u8>, // 0 = empty, 1 = pellet, 2 = power pellet
    remaining: u32,
    score: u32,
    lives: u32,
    fright: f32,
    phase: Phase,
    caught: bool,
}

impl Pacman {
    fn cell(&self, t: IVec2) -> u8 {
        self.cells[(t.y * COLS + t.x) as usize]
    }

    fn set_cell(&mut self, t: IVec2, v: u8) {
        self.cells[(t.y * COLS + t.x) as usize] = v;
    }

    fn fresh() -> Self {
        let mut cells = vec![0u8; (COLS * ROWS) as usize];
        let mut remaining = 0;
        for y in 0..ROWS {
            for x in 0..COLS {
                let t = ivec2(x, y);
                if !is_wall(t) {
                    cells[(y * COLS + x) as usize] = 1;
                    remaining += 1;
                }
            }
        }
        let mut game = Self {
            pac: TileActor::new(PAC_START, CELL, PAC_SPEED),
            ghosts: GHOST_STARTS.iter().map(|&t| TileActor::new(t, CELL, GHOST_SPEED)).collect(),
            cells,
            remaining,
            score: 0,
            lives: 3,
            fright: 0.0,
            phase: Phase::Playing,
            caught: false,
        };
        for &t in &[
            ivec2(1, 1),
            ivec2(COLS - 2, 1),
            ivec2(1, ROWS - 2),
            ivec2(COLS - 2, ROWS - 2),
        ] {
            game.set_cell(t, 2);
        }
        for &t in std::iter::once(&PAC_START).chain(GHOST_STARTS.iter()) {
            if game.cell(t) != 0 {
                game.set_cell(t, 0);
                game.remaining -= 1;
            }
        }
        game
    }

    fn restart(&mut self) {
        *self = Pacman::fresh();
        host::emit_playing();
        host::emit(&HostEvent::ScoreChanged { score: 0 });
    }

    fn reset_positions(&mut self) {
        self.pac = TileActor::new(PAC_START, CELL, PAC_SPEED);
        for (g, &t) in self.ghosts.iter_mut().zip(GHOST_STARTS.iter()) {
            *g = TileActor::new(t, CELL, GHOST_SPEED);
        }
        self.fright = 0.0;
    }

    fn add_score(&mut self, n: u32) {
        self.score += n;
        host::emit(&HostEvent::ScoreChanged { score: self.score });
    }

    fn ghost_ai(&self, g: &TileActor, frightened: bool) -> Option<Direction> {
        let tile = g.tile();
        let reverse = g.dir.map(Direction::opposite);
        let target = self.pac.pos;
        let mut best = None;
        let mut best_metric = if frightened { f32::MIN } else { f32::MAX };
        for d in ALL_DIRS {
            if Some(d) == reverse {
                continue;
            }
            let nt = tile + direction_delta(d);
            if is_wall(nt) {
                continue;
            }
            let dist = tile_center(nt).distance_squared(target);
            let better = if frightened { dist > best_metric } else { dist < best_metric };
            if better {
                best_metric = dist;
                best = Some(d);
            }
        }
        best.or(reverse)
    }
}

impl Game for Pacman {
    fn config() -> GameConfig {
        GameConfig {
            title: "Pac-Man",
            world_width: WORLD_W,
            world_height: WORLD_H,
            background: Color::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    async fn load() -> Self {
        rand::srand(macroquad::miniquad::date::now() as u64);
        host::emit(&HostEvent::Ready);
        host::emit_playing();
        Pacman::fresh()
    }

    fn update(&mut self, ctx: &mut Context) {
        if !matches!(self.phase, Phase::Playing) {
            if ctx.input.is_pressed(Action::Confirm) {
                self.restart();
            }
            return;
        }

        self.caught = false;

        if self.fright > 0.0 {
            self.fright = (self.fright - ctx.dt).max(0.0);
        }

        if let Some(d) = ctx.input.direction() {
            self.pac.want = Some(d);
        }
        self.pac.advance(ctx.dt, |t| !is_wall(t));

        let pt = self.pac.tile();
        match self.cell(pt) {
            1 => {
                self.set_cell(pt, 0);
                self.remaining -= 1;
                self.add_score(PELLET_SCORE);
            }
            2 => {
                self.set_cell(pt, 0);
                self.remaining -= 1;
                self.fright = FRIGHT_TIME;
                self.add_score(POWER_SCORE);
            }
            _ => {}
        }

        let frightened = self.fright > 0.0;
        for i in 0..self.ghosts.len() {
            let choice = self.ghost_ai(&self.ghosts[i], frightened);
            self.ghosts[i].want = choice;
            if self.ghosts[i].dir.is_none() {
                self.ghosts[i].dir = choice;
            }
            self.ghosts[i].advance(ctx.dt, |t| !is_wall(t));
        }

        let hit_size = CELL * 0.8;
        ctx.collisions.add(
            Collider::new(PAC_ID, self.pac.pos, hit_size, hit_size)
                .with_layers(LAYER_PAC, LAYER_GHOST),
        );
        for (i, g) in self.ghosts.iter().enumerate() {
            ctx.collisions.add(
                Collider::new(EntityId(i as u64 + 1), g.pos, hit_size, hit_size)
                    .with_layers(LAYER_GHOST, LAYER_PAC),
            );
        }

        if self.remaining == 0 {
            self.phase = Phase::Won;
            host::emit_game_over(self.score);
        }
    }

    fn on_collision(&mut self, a: EntityId, b: EntityId) {
        if self.caught || !matches!(self.phase, Phase::Playing) {
            return;
        }
        let ghost_id = if a == PAC_ID { b } else { a };
        let idx = (ghost_id.0 as usize).wrapping_sub(1);
        if idx >= self.ghosts.len() {
            return;
        }

        if self.fright > 0.0 {
            self.ghosts[idx] = TileActor::new(GHOST_STARTS[idx], CELL, GHOST_SPEED);
            self.add_score(GHOST_SCORE);
        } else {
            self.caught = true;
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                self.phase = Phase::Lost;
                host::emit_game_over(self.score);
            } else {
                self.reset_positions();
            }
        }
    }

    fn draw(&self, gfx: &Gfx) {
        let wall_color = Color::new(0.13, 0.16, 0.45, 1.0);
        for y in 0..ROWS {
            for x in 0..COLS {
                let t = ivec2(x, y);
                if is_wall(t) {
                    gfx.rect(x as f32 * CELL, y as f32 * CELL, CELL, CELL, wall_color);
                }
            }
        }

        for y in 0..ROWS {
            for x in 0..COLS {
                let t = ivec2(x, y);
                let c = tile_center(t);
                match self.cell(t) {
                    1 => gfx.rect(c.x - 3.0, c.y - 3.0, 6.0, 6.0, Color::new(1.0, 0.85, 0.6, 1.0)),
                    2 => gfx.rect(c.x - 7.0, c.y - 7.0, 14.0, 14.0, Color::new(1.0, 0.9, 0.4, 1.0)),
                    _ => {}
                }
            }
        }

        let frightened = self.fright > 0.0;
        let ghost_colors = [
            Color::new(0.9, 0.2, 0.2, 1.0),
            Color::new(1.0, 0.6, 0.8, 1.0),
            Color::new(0.3, 0.9, 0.9, 1.0),
        ];
        let r = CELL * 0.42;
        for (i, g) in self.ghosts.iter().enumerate() {
            let color = if frightened {
                Color::new(0.25, 0.35, 1.0, 1.0)
            } else {
                ghost_colors[i % ghost_colors.len()]
            };
            gfx.rect(g.pos.x - r, g.pos.y - r, r * 2.0, r * 2.0, color);
        }

        let pr = CELL * 0.42;
        gfx.rect(self.pac.pos.x - pr, self.pac.pos.y - pr, pr * 2.0, pr * 2.0, YELLOW);

        gfx.text(&format!("Lives: {}", self.lives), 10.0, WORLD_H - 8.0, 28.0, WHITE);

        match self.phase {
            Phase::Lost => overlay(gfx, "Game Over", "Press Enter to restart"),
            Phase::Won => overlay(gfx, "You Win!", "Press Enter to play again"),
            Phase::Playing => {}
        }
    }
}

fn overlay(gfx: &Gfx, title: &str, sub: &str) {
    gfx.rect(0.0, 0.0, WORLD_W, WORLD_H, Color::new(0.0, 0.0, 0.0, 0.6));
    gfx.text_centered(title, WORLD_W * 0.5, WORLD_H * 0.5 - 10.0, 56.0, WHITE);
    gfx.text_centered(sub, WORLD_W * 0.5, WORLD_H * 0.5 + 36.0, 26.0, Color::new(0.85, 0.85, 0.85, 1.0));
}

engine::game_main!(Pacman);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_tiles_are_corridors() {
        assert!(!is_wall(PAC_START));
        for &g in &GHOST_STARTS {
            assert!(!is_wall(g));
        }
    }

    #[test]
    fn maze_has_pellets_and_is_fully_connected() {
        let game = Pacman::fresh();
        assert!(game.remaining > 50, "expected a well-filled maze");

        let mut seen = vec![false; (COLS * ROWS) as usize];
        let mut stack = vec![PAC_START];
        seen[(PAC_START.y * COLS + PAC_START.x) as usize] = true;
        let mut count = 0;
        while let Some(t) = stack.pop() {
            count += 1;
            for d in ALL_DIRS {
                let nt = t + direction_delta(d);
                let i = (nt.y * COLS + nt.x) as usize;
                if !is_wall(nt) && !seen[i] {
                    seen[i] = true;
                    stack.push(nt);
                }
            }
        }
        let corridors = (0..ROWS)
            .flat_map(|y| (0..COLS).map(move |x| ivec2(x, y)))
            .filter(|&t| !is_wall(t))
            .count();
        assert_eq!(count, corridors, "all corridor tiles must be reachable");
    }
}
