//! Asset loading helpers. Textures are served alongside each game, so a path
//! like `"assets/snake.png"` resolves to `/games/<game>/assets/snake.png`.

use macroquad::prelude::*;

/// Load a texture with nearest-neighbor filtering (crisp pixel art).
pub async fn load_sprite(path: &str) -> Result<Texture2D, macroquad::Error> {
    let texture = load_texture(path).await?;
    texture.set_filter(FilterMode::Nearest);
    Ok(texture)
}
