use engine::macroquad::prelude::*;
use engine::macroquad::rand;
use engine::protocol::HostEvent;
use engine::{
    direction_delta, host, out_of_bounds, Action, Context, Direction, Game, GameConfig, Gfx,
};

const GRID: i32 = 20;
const CELL: f32 = 40.0;
const WORLD: f32 = GRID as f32 * CELL;
const TICK: f32 = 0.12;

enum Phase {
    Playing,
    Dead,
    Won,
}

struct Snake {
    body: Vec<IVec2>, // head first
    dir: Direction,
    next_dir: Direction,
    food: Option<IVec2>,
    timer: f32,
    score: u32,
    phase: Phase,
}

impl Snake {
    fn fresh() -> Self {
        let body = vec![ivec2(10, 10), ivec2(9, 10), ivec2(8, 10)];
        let food = random_empty(&body);
        Self {
            body,
            dir: Direction::Right,
            next_dir: Direction::Right,
            food,
            timer: 0.0,
            score: 0,
            phase: Phase::Playing,
        }
    }

    fn restart(&mut self) {
        *self = Snake::fresh();
        host::emit_playing();
        host::emit(&HostEvent::ScoreChanged { score: 0 });
    }

    fn die(&mut self) {
        self.phase = Phase::Dead;
        host::emit_game_over(self.score);
    }

    fn win(&mut self) {
        self.phase = Phase::Won;
        host::emit_game_over(self.score);
    }

    fn step(&mut self) {
        self.dir = self.next_dir;
        let head = self.body[0] + direction_delta(self.dir);

        let (hx, hy, hw, hh) = cell_rect(head, 0.0);
        if out_of_bounds(Rect::new(0.0, 0.0, WORLD, WORLD), Rect::new(hx, hy, hw, hh)) {
            self.die();
            return;
        }

        let will_grow = self.food == Some(head);
        let occupied = if will_grow {
            &self.body[..]
        } else {
            &self.body[..self.body.len() - 1]
        };
        if occupied.contains(&head) {
            self.die();
            return;
        }

        self.body.insert(0, head);
        if will_grow {
            self.score += 1;
            host::emit(&HostEvent::ScoreChanged { score: self.score });
            match random_empty(&self.body) {
                Some(pos) => self.food = Some(pos),
                None => {
                    self.food = None;
                    self.win();
                }
            }
        } else {
            self.body.pop();
        }
    }
}

fn random_empty(body: &[IVec2]) -> Option<IVec2> {
    let empties: Vec<IVec2> = (0..GRID)
        .flat_map(|y| (0..GRID).map(move |x| ivec2(x, y)))
        .filter(|c| !body.contains(c))
        .collect();
    empties
        .get(rand::gen_range(0, empties.len().max(1)))
        .copied()
}

fn cell_rect(c: IVec2, inset: f32) -> (f32, f32, f32, f32) {
    (
        c.x as f32 * CELL + inset,
        c.y as f32 * CELL + inset,
        CELL - inset * 2.0,
        CELL - inset * 2.0,
    )
}

impl Game for Snake {
    fn config() -> GameConfig {
        GameConfig {
            title: "Snake",
            world_width: WORLD,
            world_height: WORLD,
            background: Color::new(0.06, 0.07, 0.06, 1.0),
        }
    }

    async fn load() -> Self {
        rand::srand(macroquad::miniquad::date::now() as u64);
        host::emit(&HostEvent::Ready);
        host::emit_playing();
        Snake::fresh()
    }

    fn update(&mut self, ctx: &mut Context) {
        match self.phase {
            Phase::Playing => {
                if let Some(d) = ctx.input.direction() {
                    if d != self.dir.opposite() {
                        self.next_dir = d;
                    }
                }
                self.timer += ctx.dt;
                while self.timer >= TICK {
                    self.timer -= TICK;
                    self.step();
                }
            }
            Phase::Dead | Phase::Won => {
                if ctx.input.is_pressed(Action::Confirm) {
                    self.restart();
                }
            }
        }
    }

    fn draw(&self, gfx: &Gfx) {
        if let Some(food) = self.food {
            let (fx, fy, fw, fh) = cell_rect(food, CELL * 0.2);
            gfx.rect(fx, fy, fw, fh, Color::new(0.9, 0.3, 0.3, 1.0));
        }

        for (i, seg) in self.body.iter().enumerate() {
            let (x, y, w, h) = cell_rect(*seg, 2.0);
            let color = if i == 0 {
                Color::new(0.5, 0.95, 0.45, 1.0)
            } else {
                Color::new(0.3, 0.75, 0.35, 1.0)
            };
            gfx.rect(x, y, w, h, color);
        }

        match self.phase {
            Phase::Dead => overlay(gfx, "Game Over", "Press Enter to restart"),
            Phase::Won => overlay(gfx, "You Win!", "Press Enter to play again"),
            Phase::Playing => {}
        }
    }
}

fn overlay(gfx: &Gfx, title: &str, sub: &str) {
    gfx.rect(0.0, 0.0, WORLD, WORLD, Color::new(0.0, 0.0, 0.0, 0.55));
    gfx.text_centered(title, WORLD * 0.5, WORLD * 0.5 - 10.0, 64.0, WHITE);
    gfx.text_centered(
        sub,
        WORLD * 0.5,
        WORLD * 0.5 + 40.0,
        32.0,
        Color::new(0.8, 0.8, 0.8, 1.0),
    );
}

engine::game_main!(Snake);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_empty_returns_none_when_board_full() {
        let all: Vec<IVec2> = (0..GRID)
            .flat_map(|y| (0..GRID).map(move |x| ivec2(x, y)))
            .collect();
        assert!(random_empty(&all).is_none());
    }

    #[test]
    fn random_empty_returns_some_when_space_available() {
        assert!(random_empty(&[ivec2(0, 0)]).is_some());
    }

    #[test]
    fn random_empty_result_is_not_in_body() {
        let body = vec![ivec2(0, 0), ivec2(1, 0), ivec2(2, 0)];
        for _ in 0..50 {
            if let Some(pos) = random_empty(&body) {
                assert!(!body.contains(&pos));
            }
        }
    }

    #[test]
    fn step_moves_head_forward() {
        let mut s = Snake::fresh();
        let old_head = s.body[0];
        s.step();
        let expected = old_head + direction_delta(s.dir);
        assert_eq!(s.body[0], expected);
    }

    #[test]
    fn step_does_not_change_length_without_food() {
        let mut s = Snake::fresh();
        s.food = random_empty(&s.body);
        let original_len = s.body.len();
        s.next_dir = Direction::Up;
        s.step();
        assert_eq!(s.body.len(), original_len);
    }

    #[test]
    fn step_wall_collision_triggers_dead_phase() {
        let mut s = Snake::fresh();
        s.body[0] = ivec2(0, 5);
        s.dir = Direction::Left;
        s.next_dir = Direction::Left;
        s.step();
        assert!(matches!(s.phase, Phase::Dead));
    }

    #[test]
    fn step_self_collision_triggers_dead_phase() {
        let body = vec![
            ivec2(5, 5),
            ivec2(6, 5),
            ivec2(7, 5),
            ivec2(7, 6),
            ivec2(6, 6),
            ivec2(5, 6),
            ivec2(5, 5),
        ];
        let mut s = Snake::fresh();
        s.body = body;
        s.dir = Direction::Right;
        s.next_dir = Direction::Right;
        s.step();
        assert!(matches!(s.phase, Phase::Dead));
    }

    #[test]
    fn step_eating_food_grows_body_and_increments_score() {
        let mut s = Snake::fresh();
        let ahead = s.body[0] + direction_delta(s.dir);
        if ahead.x >= 0 && ahead.x < GRID && ahead.y >= 0 && ahead.y < GRID {
            s.food = Some(ahead);
            let old_len = s.body.len();
            let old_score = s.score;
            s.step();
            if matches!(s.phase, Phase::Playing) {
                assert_eq!(s.body.len(), old_len + 1);
                assert_eq!(s.score, old_score + 1);
            }
        }
    }
}
