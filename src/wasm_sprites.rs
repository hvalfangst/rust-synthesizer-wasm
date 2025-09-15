use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;
use image::GenericImageView;

#[derive(Clone)]
pub struct Sprite {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u32>,
}

impl Sprite {
    pub fn new(width: u32, height: u32, data: Vec<u32>) -> Self {
        Self { width, height, data }
    }
}

#[derive(Clone)]
pub struct WasmSprites {
    pub notes: Vec<Sprite>,
    pub numbers: Vec<Sprite>,
    pub keys: Vec<Sprite>,
    pub tangents: Vec<Sprite>,
    pub knob: Vec<Sprite>,
    pub rack: Vec<Sprite>,
    pub display_sine: Vec<Sprite>,
    pub display_square: Vec<Sprite>,
    pub octave_fader: Vec<Sprite>,
    pub bulb: Vec<Sprite>,
    pub adsr_fader: Vec<Sprite>,
}

impl WasmSprites {
    pub async fn load_all() -> Result<WasmSprites, JsValue> {
        Ok(WasmSprites {
            notes: load_sprites_from_url("assets/notes.png", 64, 48).await?,
            numbers: load_sprites_from_url("assets/numbers.png", 64, 48).await?,
            keys: load_sprites_from_url("assets/keys.png", 64, 144).await?,
            tangents: load_sprites_from_url("assets/tangents.png", 30, 96).await?,
            knob: load_sprites_from_url("assets/knob.png", 64, 48).await?,
            display_sine: load_sprites_from_url("assets/display_sine.png", 164, 51).await?,
            display_square: load_sprites_from_url("assets/display_square.png", 164, 51).await?,
            rack: load_sprites_from_url("assets/rack.png", 575, 496).await?, // Full window size
            octave_fader: load_sprites_from_url("assets/octave_fader.png", 28, 143).await?,
            bulb: load_sprites_from_url("assets/bulb.png", 12, 12).await?,
            adsr_fader: load_sprites_from_url("assets/octave_fader.png", 28, 143).await?, // Reuse for now
        })
    }
}

async fn load_sprites_from_url(url: &str, sprite_width: u32, sprite_height: u32) -> Result<Vec<Sprite>, JsValue> {
    // Fetch the image
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_str(url)).await?;

    let resp: Response = resp_value.dyn_into()?;
    let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let mut bytes = vec![0; uint8_array.length() as usize];
    uint8_array.copy_to(&mut bytes);

    // Load image using the image crate
    let img = image::load_from_memory(&bytes)
        .map_err(|e| JsValue::from_str(&format!("Failed to load image {}: {}", url, e)))?;

    let rgba_img = img.to_rgba8();
    let (map_width, map_height) = img.dimensions();

    // Calculate how many sprites fit in the image
    let sprites_x = map_width / sprite_width;
    let sprites_y = map_height / sprite_height;

    let mut sprites = Vec::new();

    // Extract each sprite
    for sprite_y in 0..sprites_y {
        for sprite_x in 0..sprites_x {
            let mut sprite_data = Vec::new();

            for y in 0..sprite_height {
                for x in 0..sprite_width {
                    let src_x = sprite_x * sprite_width + x;
                    let src_y = sprite_y * sprite_height + y;

                    if src_x < map_width && src_y < map_height {
                        let pixel = rgba_img.get_pixel(src_x, src_y);
                        let r = pixel[0] as u32;
                        let g = pixel[1] as u32;
                        let b = pixel[2] as u32;
                        let a = pixel[3] as u32;

                        // Convert RGBA to ARGB format for compatibility with original code
                        let argb_pixel = (a << 24) | (r << 16) | (g << 8) | b;
                        sprite_data.push(argb_pixel);
                    } else {
                        sprite_data.push(0); // Transparent
                    }
                }
            }

            sprites.push(Sprite::new(sprite_width, sprite_height, sprite_data));
        }
    }

    // If no sprites were extracted (single image), treat the whole image as one sprite
    if sprites.is_empty() {
        let mut sprite_data = Vec::new();
        for y in 0..map_height {
            for x in 0..map_width {
                let pixel = rgba_img.get_pixel(x, y);
                let r = pixel[0] as u32;
                let g = pixel[1] as u32;
                let b = pixel[2] as u32;
                let a = pixel[3] as u32;

                let argb_pixel = (a << 24) | (r << 16) | (g << 8) | b;
                sprite_data.push(argb_pixel);
            }
        }
        sprites.push(Sprite::new(map_width, map_height, sprite_data));
    }

    Ok(sprites)
}