//! Single-player Pac-Man on the engine. A generated pillar-lattice maze (walls
//! at even/even interior tiles, guaranteeing all corridors are connected),
//! pellets, power pellets with frightened ghosts, lives, and greedy ghost AI.
//! Uses the engine's CollisionWorld for pac<->ghost contact (Snake used grid
//! logic; this exercises the continuous collision path).

use engine::macroquad::prelude::*;
use engine::macroquad::rand;
use engine::protocol::{GameStatus, HostEvent};
use engine::{host, Action, Collider, Context, Direction, EntityId, Game, GameConfig, Gfx, Shape};

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

fn is_wall(t: IVec2) -> bool {
    let (x, y) = (t.x, t.y);
    x <= 0 || y <= 0 || x >= COLS - 1 || y >= ROWS - 1 || (x % 2 == 0 && y % 2 == 0)
}

fn delta(d: Direction) -> IVec2 {
    match d {
        Direction::Up => ivec2(0, -1),
        Direction::Down => ivec2(0, 1),
        Direction::Left => ivec2(-1, 0),
        Direction::Right => ivec2(1, 0),
    }
}

fn dvec(d: Direction) -> Vec2 {
    delta(d).as_vec2()
}

fn opposite(d: Direction) -> Direction {
    match d {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

fn tile_center(t: IVec2) -> Vec2 {
    vec2(
        t.x as f32 * CELL + CELL * 0.5,
        t.y as f32 * CELL + CELL * 0.5,
    )
}

fn world_to_tile(p: Vec2) -> IVec2 {
    ivec2((p.x / CELL).floor() as i32, (p.y / CELL).floor() as i32)
}

const ALL_DIRS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

struct Mover {
    pos: Vec2,
    dir: Option<Direction>,
    want: Option<Direction>,
    speed: f32,
}

impl Mover {
    fn at(tile: IVec2, speed: f32) -> Self {
        Self {
            pos: tile_center(tile),
            dir: None,
            want: None,
            speed,
        }
    }

    /// Returns true on the frame the mover snaps to a tile center (an
    /// intersection where AI / input may change direction).
    fn advance(&mut self, dt: f32) -> bool {
        let tile = world_to_tile(self.pos);
        let center = tile_center(tile);
        let reached = (center - self.pos).length() <= self.speed * dt;
        if reached {
            self.pos = center;
            if let Some(w) = self.want {
                if !is_wall(tile + delta(w)) {
                    self.dir = Some(w);
                    self.want = None;
                }
            }
            if let Some(d) = self.dir {
                if is_wall(tile + delta(d)) {
                    self.dir = None;
                }
            }
        }
        if let Some(d) = self.dir {
            self.pos += dvec(d) * self.speed * dt;
        }
        reached
    }
}

enum Phase {
    Playing,
    Lost,
    Won,
}

struct Pacman {
    pac: Mover,
    ghosts: Vec<Mover>,
    /// 0 = empty, 1 = pellet, 2 = power pellet.
    cells: Vec<u8>,
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
            pac: Mover::at(PAC_START, PAC_SPEED),
            ghosts: GHOST_STARTS
                .iter()
                .map(|&t| Mover::at(t, GHOST_SPEED))
                .collect(),
            cells,
            remaining,
            score: 0,
            lives: 3,
            fright: 0.0,
            phase: Phase::Playing,
            caught: false,
        };
        // Power pellets in the four corners; clear entity start tiles.
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
        host::emit(&HostEvent::StatusChanged {
            status: GameStatus::Playing,
        });
        host::emit(&HostEvent::ScoreChanged { score: 0 });
    }

    fn reset_positions(&mut self) {
        self.pac = Mover::at(PAC_START, PAC_SPEED);
        for (g, &t) in self.ghosts.iter_mut().zip(GHOST_STARTS.iter()) {
            *g = Mover::at(t, GHOST_SPEED);
        }
        self.fright = 0.0;
    }

    fn add_score(&mut self, n: u32) {
        self.score += n;
        host::emit(&HostEvent::ScoreChanged { score: self.score });
    }

    fn ghost_ai(&self, g: &Mover, frightened: bool) -> Option<Direction> {
        let tile = world_to_tile(g.pos);
        let reverse = g.dir.map(opposite);
        let target = self.pac.pos;
        let mut best = None;
        let mut best_metric = if frightened { f32::MIN } else { f32::MAX };
        for d in ALL_DIRS {
            if Some(d) == reverse {
                continue;
            }
            let nt = tile + delta(d);
            if is_wall(nt) {
                continue;
            }
            let dist = tile_center(nt).distance_squared(target);
            let better = if frightened {
                dist > best_metric
            } else {
                dist < best_metric
            };
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
        host::emit(&HostEvent::StatusChanged {
            status: GameStatus::Playing,
        });
        Pacman::fresh()
    }

    fn update(&mut self, ctx: &mut Context) {
        if !matches!(self.phase, Phase::Playing) {
            if ctx.input.is_pressed(Action::Confirm) {
                self.restart();
            }
            return;
        }

        // Per-frame guard so on_collision (dispatched after update) handles at
        // most one catch per frame, and re-arms after a respawn.
        self.caught = false;

        if self.fright > 0.0 {
            self.fright = (self.fright - ctx.dt).max(0.0);
        }

        // Pac movement + input.
        if let Some(d) = ctx.input.requested_direction() {
            self.pac.want = Some(d);
        }
        self.pac.advance(ctx.dt);

        // Eat pellet under pac.
        let pt = world_to_tile(self.pac.pos);
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

        // Ghost movement.
        let frightened = self.fright > 0.0;
        for i in 0..self.ghosts.len() {
            let tile = world_to_tile(self.ghosts[i].pos);
            let at_center =
                (tile_center(tile) - self.ghosts[i].pos).length() <= self.ghosts[i].speed * ctx.dt;
            if at_center || self.ghosts[i].dir.is_none() {
                let choice = self.ghost_ai(&self.ghosts[i], frightened);
                self.ghosts[i].want = choice;
                self.ghosts[i].dir = self.ghosts[i].dir.or(choice);
            }
            self.ghosts[i].advance(ctx.dt);
        }

        // Register colliders for the engine to report pac<->ghost contact.
        ctx.collisions.add(
            Collider::new(PAC_ID, self.pac.pos, Shape::Circle { radius: CELL * 0.4 })
                .with_layers(LAYER_PAC, LAYER_GHOST),
        );
        for (i, g) in self.ghosts.iter().enumerate() {
            ctx.collisions.add(
                Collider::new(
                    EntityId(i as u64 + 1),
                    g.pos,
                    Shape::Circle { radius: CELL * 0.4 },
                )
                .with_layers(LAYER_GHOST, LAYER_PAC),
            );
        }

        if self.remaining == 0 {
            self.phase = Phase::Won;
            host::emit(&HostEvent::GameOver { score: self.score });
            host::emit(&HostEvent::StatusChanged {
                status: GameStatus::GameOver,
            });
        }
    }

    fn on_collision(&mut self, a: EntityId, b: EntityId) {
        if self.caught || !matches!(self.phase, Phase::Playing) {
            return;
        }
        // Identify the ghost (the non-pac id).
        let ghost_id = if a == PAC_ID { b } else { a };
        let idx = (ghost_id.0 as usize).wrapping_sub(1);
        if idx >= self.ghosts.len() {
            return;
        }

        if self.fright > 0.0 {
            // Eat the ghost: send it home.
            self.ghosts[idx] = Mover::at(GHOST_STARTS[idx], GHOST_SPEED);
            self.add_score(GHOST_SCORE);
        } else {
            self.caught = true;
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                self.phase = Phase::Lost;
                host::emit(&HostEvent::GameOver { score: self.score });
                host::emit(&HostEvent::StatusChanged {
                    status: GameStatus::GameOver,
                });
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
                    1 => gfx.circle(c.x, c.y, 3.0, Color::new(1.0, 0.85, 0.6, 1.0)),
                    2 => gfx.circle(c.x, c.y, 7.0, Color::new(1.0, 0.9, 0.4, 1.0)),
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
        for (i, g) in self.ghosts.iter().enumerate() {
            let color = if frightened {
                Color::new(0.25, 0.35, 1.0, 1.0)
            } else {
                ghost_colors[i % ghost_colors.len()]
            };
            gfx.circle(g.pos.x, g.pos.y, CELL * 0.42, color);
        }

        gfx.circle(self.pac.pos.x, self.pac.pos.y, CELL * 0.42, YELLOW);

        gfx.text(
            &format!("Lives: {}", self.lives),
            10.0,
            WORLD_H - 8.0,
            28.0,
            WHITE,
        );

        match self.phase {
            Phase::Lost => overlay(gfx, "Game Over", "Press Enter or tap to restart"),
            Phase::Won => overlay(gfx, "You Win!", "Press Enter or tap to play again"),
            Phase::Playing => {}
        }
    }
}

fn overlay(gfx: &Gfx, title: &str, sub: &str) {
    gfx.rect(0.0, 0.0, WORLD_W, WORLD_H, Color::new(0.0, 0.0, 0.0, 0.6));
    gfx.text_centered(title, WORLD_W * 0.5, WORLD_H * 0.5 - 10.0, 56.0, WHITE);
    gfx.text_centered(
        sub,
        WORLD_W * 0.5,
        WORLD_H * 0.5 + 36.0,
        26.0,
        Color::new(0.85, 0.85, 0.85, 1.0),
    );
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

        // Flood fill from pac start over corridors; every corridor tile must be
        // reachable (the pillar lattice guarantees this).
        let mut seen = vec![false; (COLS * ROWS) as usize];
        let mut stack = vec![PAC_START];
        seen[(PAC_START.y * COLS + PAC_START.x) as usize] = true;
        let mut count = 0;
        while let Some(t) = stack.pop() {
            count += 1;
            for d in ALL_DIRS {
                let nt = t + delta(d);
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
