pub mod assets;
pub mod collision;
pub mod gfx;
pub mod host;
pub mod input;
pub mod movement;
pub mod types;

pub use collision::{out_of_bounds, Collider, CollisionWorld, Side};
pub use gfx::Gfx;
pub use input::{direction_delta, Action, Direction, Input};
pub use movement::TileActor;
pub use types::EntityId;

pub use macroquad;
pub use protocol;

use macroquad::prelude::*;

pub struct GameConfig {
    pub title: &'static str,
    pub world_width: f32,
    pub world_height: f32,
    pub background: Color,
}

pub struct Context<'a> {
    pub dt: f32,
    pub time: f64,
    pub bounds: Rect,
    pub input: &'a Input,
    pub collisions: &'a mut CollisionWorld,
}

#[allow(async_fn_in_trait)]
pub trait Game: Sized + 'static {
    fn config() -> GameConfig;
    async fn load() -> Self;
    fn update(&mut self, ctx: &mut Context);
    fn draw(&self, gfx: &Gfx);
    fn on_collision(&mut self, _a: EntityId, _b: EntityId) {}
    fn on_boundary(&mut self, _entity: EntityId, _side: Side) {}
}

pub async fn run<G: Game>() {
    let cfg = G::config();
    let gfx = Gfx::new(cfg.world_width, cfg.world_height);
    let input = Input::new();
    let mut world = CollisionWorld::new(gfx.bounds());

    let mut game = G::load().await;

    loop {
        world.clear();
        let mut ctx = Context {
            dt: get_frame_time(),
            time: get_time(),
            bounds: gfx.bounds(),
            input: &input,
            collisions: &mut world,
        };
        game.update(&mut ctx);

        world.for_each_collision(|a, b| game.on_collision(a, b));
        world.for_each_boundary(|entity, side| game.on_boundary(entity, side));

        clear_background(cfg.background);
        gfx.begin();
        game.draw(&gfx);
        gfx.end();

        next_frame().await;
    }
}

#[macro_export]
macro_rules! game_main {
    ($game:ty) => {
        fn __engine_window_conf() -> $crate::macroquad::window::Conf {
            let cfg = <$game as $crate::Game>::config();
            $crate::macroquad::window::Conf {
                window_title: cfg.title.to_string(),
                high_dpi: true,
                ..::core::default::Default::default()
            }
        }

        #[$crate::macroquad::main(__engine_window_conf)]
        async fn main() {
            $crate::run::<$game>().await;
        }
    };
}
