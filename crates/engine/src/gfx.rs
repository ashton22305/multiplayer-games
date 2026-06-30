//! Rendering surface with a fixed virtual resolution. Games draw in world
//! units; the world is letterboxed into the canvas, so everything scales with
//! the window/iframe size without distortion.

use macroquad::prelude::*;

pub struct Gfx {
    world: Vec2,
}

impl Gfx {
    pub fn new(world_width: f32, world_height: f32) -> Self {
        Self {
            world: vec2(world_width, world_height),
        }
    }

    /// The world rectangle, `(0, 0)` to `(world_width, world_height)`.
    pub fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.world.x, self.world.y)
    }

    /// Activate the world camera for this frame. The runtime calls this before
    /// [`crate::Game::draw`]; games may call [`Gfx::end`] to draw screen-space HUD.
    pub fn begin(&self) {
        set_camera(&self.camera());
    }

    /// Restore the default screen-space camera.
    pub fn end(&self) {
        set_default_camera();
    }

    fn camera(&self) -> Camera2D {
        let sw = screen_width().max(1.0);
        let sh = screen_height().max(1.0);
        let screen_aspect = sw / sh;
        let world_aspect = self.world.x / self.world.y;
        // Expand the displayed rect to the screen's aspect ratio so the world
        // is centered with letterbox bars rather than stretched.
        let rect = if screen_aspect > world_aspect {
            let w = self.world.y * screen_aspect;
            Rect::new(-(w - self.world.x) * 0.5, 0.0, w, self.world.y)
        } else {
            let h = self.world.x / screen_aspect;
            Rect::new(0.0, -(h - self.world.y) * 0.5, self.world.x, h)
        };
        // miniquad's WebGL backend flips Y when blitting to the canvas, so using
        // from_display_rect (zoom.y = -2/h) double-flips and puts y=0 at the bottom.
        // Positive zoom.y lets miniquad's single flip land y=0 at the top.
        Camera2D {
            zoom: vec2(2.0 / rect.w, 2.0 / rect.h),
            target: vec2(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0),
            ..Default::default()
        }
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        draw_rectangle(x, y, w, h, color);
    }

    pub fn rect_lines(&self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
        draw_rectangle_lines(x, y, w, h, thickness, color);
    }

    pub fn circle(&self, x: f32, y: f32, r: f32, color: Color) {
        draw_circle(x, y, r, color);
    }

    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        draw_line(x1, y1, x2, y2, thickness, color);
    }

    /// Draw a texture at `pos` scaled to `size` (world units), rotated by
    /// `rotation` radians. Scaling is what makes sprites resize with the canvas.
    pub fn sprite(&self, tex: &Texture2D, pos: Vec2, size: Vec2, rotation: f32) {
        draw_texture_ex(
            tex,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                rotation,
                ..Default::default()
            },
        );
    }

    pub fn text(&self, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
        draw_text(text, x, y, font_size, color);
    }

    /// Draw text horizontally centered on the point `(cx, y)`.
    pub fn text_centered(&self, text: &str, cx: f32, y: f32, font_size: f32, color: Color) {
        let dim = measure_text(text, None, font_size as u16, 1.0);
        draw_text(text, cx - dim.width * 0.5, y, font_size, color);
    }
}
