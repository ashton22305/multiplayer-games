//! Pipeline-proof demo: a circle bouncing off the world boundaries. Exercises
//! the build -> wasm -> iframe -> Vue path, virtual-resolution scaling, the
//! collision/boundary system, and the host event bridge.

use engine::macroquad::prelude::*;
use engine::protocol::HostEvent;
use engine::{host, Collider, Context, EntityId, Game, GameConfig, Gfx, Shape, Side};

const BALL: EntityId = EntityId(1);
const WORLD: f32 = 800.0;

struct Demo {
    pos: Vec2,
    vel: Vec2,
    radius: f32,
    bounces: u32,
}

impl Game for Demo {
    fn config() -> GameConfig {
        GameConfig {
            title: "Engine Demo",
            world_width: WORLD,
            world_height: WORLD,
            background: Color::new(0.05, 0.05, 0.08, 1.0),
        }
    }

    async fn load() -> Self {
        host::emit(&HostEvent::Ready);
        Self {
            pos: vec2(400.0, 400.0),
            vel: vec2(260.0, 200.0),
            radius: 36.0,
            bounces: 0,
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        self.pos += self.vel * ctx.dt;
        // Register the ball so the runtime reports boundary crossings.
        ctx.collisions.add(Collider::new(
            BALL,
            self.pos,
            Shape::Circle {
                radius: self.radius,
            },
        ));
    }

    fn on_boundary(&mut self, _entity: EntityId, side: Side) {
        match side {
            Side::Left => {
                self.pos.x = self.radius;
                self.vel.x = self.vel.x.abs();
            }
            Side::Right => {
                self.pos.x = WORLD - self.radius;
                self.vel.x = -self.vel.x.abs();
            }
            Side::Top => {
                self.pos.y = self.radius;
                self.vel.y = self.vel.y.abs();
            }
            Side::Bottom => {
                self.pos.y = WORLD - self.radius;
                self.vel.y = -self.vel.y.abs();
            }
        }
        self.bounces += 1;
        host::emit(&HostEvent::ScoreChanged {
            score: self.bounces,
        });
    }

    fn draw(&self, gfx: &Gfx) {
        let b = gfx.bounds();
        gfx.rect_lines(b.x, b.y, b.w, b.h, 4.0, DARKGRAY);
        gfx.circle(self.pos.x, self.pos.y, self.radius, GREEN);
    }
}

engine::game_main!(Demo);
