use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use std::collections::HashMap;

mod wasm_audio;
mod wasm_sprites;

use wasm_audio::{WasmAudioEngine, WaveformType};
use wasm_sprites::{WasmSprites, Sprite};

// Graphics constants from original
const WINDOW_WIDTH: usize = 575;
const WINDOW_HEIGHT: usize = 496;

const KEY_IDLE: usize = 0;
const KEY_PRESSED: usize = 1;
const TANGENT_IDLE: usize = 0;
const TANGENT_PRESSED: usize = 1;

const WAVEFORM_SINE: usize = 0;
const WAVEFORM_SQUARE: usize = 1;

// Note sprite constants
const NOTE_A: usize = 0;
const NOTE_A_SHARP: usize = 1;
const NOTE_B: usize = 2;
const NOTE_C: usize = 3;
const NOTE_C_SHARP: usize = 4;
const NOTE_D: usize = 5;
const NOTE_D_SHARP: usize = 6;
const NOTE_E: usize = 7;
const NOTE_F: usize = 8;
const NOTE_F_SHARP: usize = 9;
const NOTE_G: usize = 10;
const NOTE_G_SHARP: usize = 11;

const WAVEFORM_TRIANGLE: usize = 2;
const WAVEFORM_SAWTOOTH: usize = 3;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WasmSynthesizer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    audio_engine: WasmAudioEngine,
    sprites: Option<WasmSprites>,
    pixel_buffer: Vec<u32>,

    // Synthesizer state
    current_track: usize,
    current_waveform: usize,
    current_octave: i32,
    volume: f32,
    pressed_keys: HashMap<String, bool>,

    // Track states
    tracks: Vec<SynthTrack>,
}

#[derive(Clone)]
struct SynthTrack {
    waveform: usize,
    octave: i32,
    volume: f32,
}

impl SynthTrack {
    fn new() -> Self {
        Self {
            waveform: WAVEFORM_SQUARE,
            octave: 4,
            volume: 0.8,
        }
    }
}

#[wasm_bindgen]
impl WasmSynthesizer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmSynthesizer, JsValue> {
        let document = web_sys::window()
            .ok_or("No global window object")?
            .document()
            .ok_or("No document found")?;

        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;

        canvas.set_width(WINDOW_WIDTH as u32);
        canvas.set_height(WINDOW_HEIGHT as u32);
        canvas.set_id("synthesizer-canvas");

        let context = canvas
            .get_context("2d")?
            .ok_or("Failed to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let audio_engine = WasmAudioEngine::new();
        let pixel_buffer = vec![0xFF000000u32; WINDOW_WIDTH * WINDOW_HEIGHT];

        // Initialize tracks
        let tracks = vec![SynthTrack::new(); 4]; // 4 tracks like original

        Ok(WasmSynthesizer {
            canvas,
            context,
            audio_engine,
            sprites: None,
            pixel_buffer,
            current_track: 0,
            current_waveform: WAVEFORM_SQUARE,
            current_octave: 4,
            volume: 0.8,
            pressed_keys: HashMap::new(),
            tracks,
        })
    }

    #[wasm_bindgen]
    pub fn get_canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }

    #[wasm_bindgen]
    pub async fn init_sprites(&mut self) -> Result<(), JsValue> {
        let sprites = WasmSprites::load_all().await?;
        self.sprites = Some(sprites);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn init_audio(&mut self) -> Result<(), JsValue> {
        self.audio_engine.init()
    }

    #[wasm_bindgen]
    pub fn play_note(&mut self, note_name: &str) -> Result<(), JsValue> {
        let track = &self.tracks[self.current_track];
        if let Ok(frequency) = self.note_to_frequency(note_name, track.octave) {
            let waveform = match track.waveform {
                WAVEFORM_SINE => WaveformType::Sine,
                WAVEFORM_SQUARE => WaveformType::Square,
                WAVEFORM_TRIANGLE => WaveformType::Triangle,
                WAVEFORM_SAWTOOTH => WaveformType::Sawtooth,
                _ => WaveformType::Square,
            };

            self.audio_engine.play_note(frequency, &waveform, track.volume, self.current_track)?;
            self.pressed_keys.insert(note_name.to_string(), true);
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn stop_note(&mut self, note_name: &str) -> Result<(), JsValue> {
        self.audio_engine.stop_note(self.current_track)?;
        self.pressed_keys.remove(note_name);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn handle_key_down(&mut self, key: &str) {
        let note = self.map_key_to_note(key);
        if let Some(note_name) = note {
            if !self.pressed_keys.contains_key(note_name) {
                let _ = self.play_note(note_name);
            }
        }

        // Handle interface controls
        match key {
            "Digit1" => self.set_waveform(WAVEFORM_SINE),
            "Digit2" => self.set_waveform(WAVEFORM_SQUARE),
            "Digit3" => self.set_waveform(WAVEFORM_TRIANGLE),
            "Digit4" => self.set_waveform(WAVEFORM_SAWTOOTH),
            "ArrowUp" => self.adjust_octave(1),
            "ArrowDown" => self.adjust_octave(-1),
            _ => {}
        }
    }

    #[wasm_bindgen]
    pub fn handle_key_up(&mut self, key: &str) {
        let note = self.map_key_to_note(key);
        if let Some(note_name) = note {
            let _ = self.stop_note(note_name);
        }
    }

    fn map_key_to_note(&self, key: &str) -> Option<&'static str> {
        match key {
            "KeyA" => Some("C"),
            "KeyW" => Some("C#"),
            "KeyS" => Some("D"),
            "KeyE" => Some("D#"),
            "KeyD" => Some("E"),
            "KeyF" => Some("F"),
            "KeyT" => Some("F#"),
            "KeyG" => Some("G"),
            "KeyY" => Some("G#"),
            "KeyH" => Some("A"),
            "KeyU" => Some("A#"),
            "KeyJ" => Some("B"),
            _ => None,
        }
    }

    fn set_waveform(&mut self, waveform: usize) {
        self.tracks[self.current_track].waveform = waveform;
        self.current_waveform = waveform;
    }

    fn adjust_octave(&mut self, delta: i32) {
        let new_octave = (self.tracks[self.current_track].octave + delta).clamp(1, 8);
        self.tracks[self.current_track].octave = new_octave;
        self.current_octave = new_octave;
    }

    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<(), JsValue> {
        // Clear buffer
        self.pixel_buffer.fill(0xFF000000);

        if self.sprites.is_some() {
            // Clone sprites to avoid borrowing issues
            let sprites = self.sprites.clone().unwrap();

            // Draw the main rack/background
            self.draw_sprite(0, 0, &sprites.rack[0]);

            // Draw waveform display based on current waveform
            let display_x = 164; // 1 * sprite width
            let display_y = 221; // 4 * sprite height + 17

            match self.current_waveform {
                WAVEFORM_SINE => self.draw_sprite(display_x, display_y, &sprites.display_sine[0]),
                WAVEFORM_SQUARE => self.draw_sprite(display_x, display_y, &sprites.display_square[0]),
                _ => self.draw_sprite(display_x, display_y, &sprites.display_sine[0]), // Default
            }

            // Draw keyboard keys
            self.draw_keyboard_internal(&sprites);

            // Draw note display (letters for pressed keys)
            self.draw_note_display(&sprites);

            // Draw octave display (using numbers)
            self.draw_octave_display_internal(&sprites);

            // Draw control knobs/faders
            self.draw_controls_internal(&sprites);
        }

        // Convert to ImageData and draw to canvas
        let image_data = self.get_image_data()?;
        self.context.put_image_data(&image_data, 0.0, 0.0)?;

        Ok(())
    }

    fn draw_sprite(&mut self, x: usize, y: usize, sprite: &Sprite) {
        for dy in 0..sprite.height as usize {
            for dx in 0..sprite.width as usize {
                let src_idx = dy * sprite.width as usize + dx;
                let dst_x = x + dx;
                let dst_y = y + dy;

                if dst_x < WINDOW_WIDTH && dst_y < WINDOW_HEIGHT && src_idx < sprite.data.len() {
                    let dst_idx = dst_y * WINDOW_WIDTH + dst_x;
                    if dst_idx < self.pixel_buffer.len() {
                        let pixel = sprite.data[src_idx];
                        // Only draw non-transparent pixels
                        if (pixel & 0xFF000000) != 0 {
                            self.pixel_buffer[dst_idx] = pixel;
                        }
                    }
                }
            }
        }
    }

    fn draw_keyboard_internal(&mut self, sprites: &WasmSprites) {
        let key_positions = [
            (1, "C"), (2, "D"), (3, "E"), (4, "F"), (5, "G"), (6, "A"), (7, "B")
        ];

        // Draw white keys (starting at position 1, not 0)
        let key_y = 2 * sprites.keys[0].height as usize; // 2 * key height
        for (pos, note) in &key_positions {
            let x = pos * sprites.keys[0].width as usize; // Use actual sprite width
            let is_pressed = self.pressed_keys.contains_key(*note);
            let sprite_idx = if is_pressed { KEY_PRESSED } else { KEY_IDLE };

            if sprite_idx < sprites.keys.len() {
                self.draw_sprite(x, key_y, &sprites.keys[sprite_idx]);
            }
        }

        // Draw black keys (tangents) using original positioning logic
        let tangent_key_positions = [
            (2, "C#"), (3, "D#"), (5, "F#"), (6, "G#"), (7, "A#")
        ];

        let key_width = sprites.keys[0].width as i32;
        let tangent_width = sprites.tangents[0].width as i32;
        let tangent_y = key_y; // Same Y as white keys

        for (pos, note) in &tangent_key_positions {
            let is_pressed = self.pressed_keys.contains_key(*note);
            let sprite_idx = if is_pressed { TANGENT_PRESSED } else { TANGENT_IDLE };

            if sprite_idx < sprites.tangents.len() {
                // Calculate x position: center between keys, offset by tangent width
                let x = (pos * key_width) - (tangent_width / 2);
                let x_usize = if x >= 0 { x as usize } else { 0 };

                self.draw_sprite(x_usize, tangent_y, &sprites.tangents[sprite_idx]);
            }
        }
    }

    fn draw_octave_display_internal(&mut self, _sprites: &WasmSprites) {
        // Octave number display removed as requested
    }

    fn draw_controls_internal(&mut self, sprites: &WasmSprites) {
        // Draw octave fader at correct position (matching original layout)
        if !sprites.octave_fader.is_empty() && !sprites.keys.is_empty() {
            let octave_index = (self.current_octave - 1).clamp(0, sprites.octave_fader.len() as i32 - 1) as usize;
            let x = 8 * sprites.keys[0].width as usize + 5;  // 8 key widths + 5 pixels
            let y = 2 * sprites.keys[0].height as usize;     // 2 key heights down

            self.draw_sprite(x, y, &sprites.octave_fader[octave_index]);
        }

        // Draw knobs for other controls (volume, etc.)
        if !sprites.knob.is_empty() {
            // Volume knob position (adjust based on original layout)
            let knob_x = 6 * sprites.knob[0].width as usize;
            let knob_y = 5 * sprites.knob[0].height as usize - 10;
            self.draw_sprite(knob_x, knob_y, &sprites.knob[0]);
        }
    }

    fn get_image_data(&self) -> Result<ImageData, JsValue> {
        let rgba = self.pixel_buffer_to_rgba();
        ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&rgba),
            WINDOW_WIDTH as u32,
            WINDOW_HEIGHT as u32,
        )
    }

    fn pixel_buffer_to_rgba(&self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.pixel_buffer.len() * 4);
        for &pixel in &self.pixel_buffer {
            rgba.push(((pixel >> 16) & 0xFF) as u8); // R
            rgba.push(((pixel >> 8) & 0xFF) as u8);  // G
            rgba.push((pixel & 0xFF) as u8);         // B
            rgba.push(((pixel >> 24) & 0xFF) as u8); // A
        }
        rgba
    }

    fn note_to_frequency(&self, note_name: &str, octave: i32) -> Result<f32, &'static str> {
        let base_frequency = match note_name {
            "C" => 261.63,
            "C#" => 277.18,
            "D" => 293.66,
            "D#" => 311.13,
            "E" => 329.63,
            "F" => 349.23,
            "F#" => 369.99,
            "G" => 392.00,
            "G#" => 415.30,
            "A" => 440.0,
            "A#" => 466.16,
            "B" => 493.88,
            _ => return Err("Invalid note name"),
        };

        Ok(base_frequency * 2.0_f32.powi(octave - 4))
    }

    #[wasm_bindgen]
    pub fn get_current_octave(&self) -> i32 {
        self.current_octave
    }

    #[wasm_bindgen]
    pub fn get_current_waveform(&self) -> usize {
        self.current_waveform
    }

    #[wasm_bindgen]
    pub fn set_track_volume(&mut self, volume: f32) {
        self.tracks[self.current_track].volume = volume.clamp(0.0, 1.0);
        self.volume = volume;
    }

    fn get_note_sprite_index(&self, note_name: &str) -> Option<usize> {
        match note_name {
            "A" => Some(NOTE_A),
            "A#" => Some(NOTE_A_SHARP),
            "B" => Some(NOTE_B),
            "C" => Some(NOTE_C),
            "C#" => Some(NOTE_C_SHARP),
            "D" => Some(NOTE_D),
            "D#" => Some(NOTE_D_SHARP),
            "E" => Some(NOTE_E),
            "F" => Some(NOTE_F),
            "F#" => Some(NOTE_F_SHARP),
            "G" => Some(NOTE_G),
            "G#" => Some(NOTE_G_SHARP),
            _ => None,
        }
    }

    fn draw_note_display(&mut self, sprites: &WasmSprites) {
        // Find the first pressed key to display its note
        for (note_name, _) in &self.pressed_keys {
            if let Some(note_sprite_index) = self.get_note_sprite_index(note_name) {
                if note_sprite_index < sprites.notes.len() {
                    // Position for note display (leftmost display area)
                    // Based on original layout: 1 * sprite_width, 5 * sprite_height - 15
                    let x = 1 * sprites.notes[0].width as usize;
                    let y = 5 * sprites.notes[0].height as usize - 15;
                    self.draw_sprite(x, y, &sprites.notes[note_sprite_index]);
                }
                break; // Only show the first pressed note
            }
        }
    }
}