// Simple input handling for WASM synthesizer

pub struct WasmInputHandler {
    // Future: could track key states, mouse positions, etc.
}

impl WasmInputHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn map_key_to_note(key_code: &str) -> Option<&'static str> {
        match key_code {
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
}