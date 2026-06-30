//! A small Rust + WebAssembly 2D game engine built on macroquad.
//!
//! Games implement [`Game`] and hand off to the runtime with [`game_main!`].
//! The runtime owns the frame loop, virtual-resolution rendering ([`Gfx`]),
//! input ([`Input`]), and (later) collision and networking, so games only
//! describe their own state and behaviour.

pub mod assets;
pub mod collision;
pub mod gfx;
pub mod host;
pub mod input;
pub mod net;
pub mod types;

pub use collision::{Collider, CollisionWorld, Shape};
pub use gfx::Gfx;
pub use input::{Action, Direction, Input};
pub use types::{EntityId, Side};

// Re-export so games and the `game_main!` macro can reach macroquad and the
// shared protocol types without declaring them as direct dependencies.
pub use macroquad;
pub use protocol;

use macroquad::prelude::*;

/// Static configuration a game provides to the runtime.
pub struct GameConfig {
    pub title: &'static str,
    /// Virtual world size, in world units. The world is letterboxed into the
    /// canvas, so these stay constant regardless of the on-screen pixel size.
    pub world_width: f32,
    pub world_height: f32,
    /// Color used to clear the canvas each frame (including letterbox bars).
    pub background: Color,
}

/// Per-frame context handed to [`Game::update`].
pub struct Context<'a> {
    /// Seconds since the previous frame.
    pub dt: f32,
    /// Seconds since the game started.
    pub time: f64,
    /// The world rectangle; use for boundary checks.
    pub bounds: Rect,
    pub input: &'a Input,
    /// Register colliders here during `update`; the runtime reports overlaps and
    /// boundary crossings afterwards via `on_collision` / `on_boundary`.
    pub collisions: &'a mut CollisionWorld,
}

/// A game implemented against the engine.
// The runtime is single-threaded (browser wasm), so the `load` future needs no
// `Send` bound; silence the lint that warns about that.
#[allow(async_fn_in_trait)]
pub trait Game: Sized + 'static {
    /// Static configuration (title, world size, background). Called before the
    /// window is created, so it must not touch engine state.
    fn config() -> GameConfig;

    /// Asynchronously construct the game, loading any assets (textures, etc.).
    async fn load() -> Self;

    /// Advance the simulation by `ctx.dt` seconds.
    fn update(&mut self, ctx: &mut Context);

    /// Render the current state. The world camera is already active.
    fn draw(&self, gfx: &Gfx);

    /// Two registered colliders began overlapping this frame.
    fn on_collision(&mut self, _a: EntityId, _b: EntityId) {}

    /// A registered entity crossed a world boundary this frame.
    fn on_boundary(&mut self, _entity: EntityId, _side: Side) {}

    /// A message arrived from the server.
    fn on_message(&mut self, _msg: protocol::ServerMsg) {}
}

/// Drive a [`Game`] through the frame loop. Invoked by [`game_main!`].
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

/// Generate the wasm entry point for a game type implementing [`Game`].
///
/// ```ignore
/// engine::game_main!(MyGame);
/// ```
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
