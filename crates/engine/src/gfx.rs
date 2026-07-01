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

    pub fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.world.x, self.world.y)
    }

    pub fn begin(&self) {
        // miniquad's WebGL backend flips Y; positive zoom.y corrects this so y=0 is at the top.
        set_camera(&Camera2D {
            zoom: vec2(2.0 / self.world.x, 2.0 / self.world.y),
            target: vec2(self.world.x / 2.0, self.world.y / 2.0),
            ..Default::default()
        });
    }

    pub fn end(&self) {
        set_default_camera();
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        draw_rectangle(x, y, w, h, color);
    }

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

    pub fn text_centered(&self, text: &str, cx: f32, y: f32, font_size: f32, color: Color) {
        let dim = measure_text(text, None, font_size as u16, 1.0);
        draw_text(text, cx - dim.width * 0.5, y, font_size, color);
    }
}
