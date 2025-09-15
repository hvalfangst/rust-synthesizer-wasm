use wasm_bindgen::prelude::*;
use web_sys::ImageData;
use crate::{Track, wasm_audio::WaveformType};

pub struct WasmRenderer {
    width: usize,
    height: usize,
    pixel_buffer: Vec<u32>,
}

impl WasmRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        let pixel_buffer = vec![0xFF1a1a2e; width * height]; // Dark blue background
        Self {
            width,
            height,
            pixel_buffer,
        }
    }

    pub fn clear(&mut self) {
        self.pixel_buffer.fill(0xFF1a1a2e); // Dark blue background
    }

    pub fn draw_keyboard(&mut self) {
        let key_width = 50;
        let key_height = 200;
        let black_key_width = 30;
        let black_key_height = 120;

        let start_x = 150;
        let start_y = self.height - key_height - 50;

        // Draw white keys
        let white_keys = ["C", "D", "E", "F", "G", "A", "B"];
        for (i, _key) in white_keys.iter().enumerate() {
            let x = start_x + i * key_width;
            self.draw_rect(x, start_y, key_width - 2, key_height, 0xFFFFFFFF);
            self.draw_rect(x, start_y, key_width - 2, 2, 0xFF000000); // Top border
            self.draw_rect(x, start_y, 2, key_height, 0xFF000000);   // Left border
            self.draw_rect(x + key_width - 2, start_y, 2, key_height, 0xFF000000); // Right border
            self.draw_rect(x, start_y + key_height - 2, key_width - 2, 2, 0xFF000000); // Bottom border
        }

        // Draw black keys
        let black_key_positions = [0.7, 1.7, 3.7, 4.7, 5.7]; // Relative positions
        for &pos in &black_key_positions {
            let x = start_x + (pos * key_width as f32) as usize - black_key_width / 2;
            self.draw_rect(x, start_y, black_key_width, black_key_height, 0xFF000000);
        }

        // Draw keyboard labels
        self.draw_text("A", start_x + 20, start_y + key_height - 20, 0xFF000000);
        self.draw_text("S", start_x + 70, start_y + key_height - 20, 0xFF000000);
        self.draw_text("D", start_x + 120, start_y + key_height - 20, 0xFF000000);
        self.draw_text("F", start_x + 170, start_y + key_height - 20, 0xFF000000);
        self.draw_text("G", start_x + 220, start_y + key_height - 20, 0xFF000000);
        self.draw_text("H", start_x + 270, start_y + key_height - 20, 0xFF000000);
        self.draw_text("J", start_x + 320, start_y + key_height - 20, 0xFF000000);
    }

    pub fn draw_track_info(&mut self, tracks: &[Track]) {
        let y = 30;
        for (i, track) in tracks.iter().enumerate() {
            let x = 40 + i * 300;

            // Track background
            self.draw_rect(x, y, 250, 100, 0xFF333333);
            self.draw_rect(x, y, 250, 2, 0xFF40e0d0); // Top border
            self.draw_rect(x, y, 2, 100, 0xFF40e0d0);  // Left border
            self.draw_rect(x + 248, y, 2, 100, 0xFF40e0d0); // Right border
            self.draw_rect(x, y + 98, 250, 2, 0xFF40e0d0); // Bottom border

            // Track title
            let title = format!("Track {}", i + 1);
            self.draw_text(&title, x + 10, y + 20, 0xFF40e0d0);

            // Waveform info
            let waveform_str = format!("Wave: {:?}", track.waveform);
            self.draw_text(&waveform_str, x + 10, y + 40, 0xFFCCCCCC);


            // Volume info
            let volume_str = format!("Vol: {:.1}", track.volume);
            self.draw_text(&volume_str, x + 10, y + 60, 0xFFCCCCCC);
        }
    }

    fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u32) {
        for dy in 0..height {
            for dx in 0..width {
                let px = x + dx;
                let py = y + dy;
                if px < self.width && py < self.height {
                    let index = py * self.width + px;
                    if index < self.pixel_buffer.len() {
                        self.pixel_buffer[index] = color;
                    }
                }
            }
        }
    }

    fn draw_text(&mut self, text: &str, x: usize, y: usize, color: u32) {
        // Simple text rendering - just draw a small rectangle for each character
        let char_width = 8;
        let char_height = 12;

        for (i, _ch) in text.chars().enumerate() {
            let char_x = x + i * char_width;
            if char_x < self.width && y < self.height {
                // Draw a simple character placeholder
                self.draw_rect(char_x, y, char_width - 1, char_height, color);
            }
        }
    }

    pub fn get_image_data(&self) -> Result<ImageData, JsValue> {
        let rgba = self.pixel_buffer_to_rgba();
        ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&rgba),
            self.width as u32,
            self.height as u32,
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
}