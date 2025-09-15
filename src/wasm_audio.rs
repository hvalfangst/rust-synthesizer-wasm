use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, OscillatorNode, GainNode};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum WaveformType {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

pub struct WasmAudioEngine {
    audio_context: Option<AudioContext>,
    active_oscillators: HashMap<usize, (OscillatorNode, GainNode)>,
    master_gain: Option<GainNode>,
}

impl WasmAudioEngine {
    pub fn new() -> Self {
        Self {
            audio_context: None,
            active_oscillators: HashMap::new(),
            master_gain: None,
        }
    }

    pub fn init(&mut self) -> Result<(), JsValue> {
        let audio_context = AudioContext::new()?;

        // Create master gain node
        let master_gain = audio_context.create_gain()?;
        master_gain.gain().set_value(0.3); // Set master volume
        master_gain.connect_with_audio_node(&audio_context.destination())?;

        self.audio_context = Some(audio_context);
        self.master_gain = Some(master_gain);

        Ok(())
    }

    pub fn play_note(&mut self, frequency: f32, waveform: &WaveformType, volume: f32, track_id: usize) -> Result<(), JsValue> {
        if let (Some(ref audio_context), Some(ref master_gain)) = (&self.audio_context, &self.master_gain) {
            // Stop any existing oscillator for this track
            if let Some((old_osc, _)) = self.active_oscillators.remove(&track_id) {
                old_osc.stop()?;
            }

            // Create new oscillator
            let oscillator = audio_context.create_oscillator()?;
            let gain_node = audio_context.create_gain()?;

            // Set oscillator properties
            oscillator.set_type(self.waveform_to_web_sys(waveform)?);
            oscillator.frequency().set_value(frequency);

            // Set gain
            gain_node.gain().set_value(volume);

            // Connect audio graph: oscillator -> gain -> master_gain -> destination
            oscillator.connect_with_audio_node(&gain_node)?;
            gain_node.connect_with_audio_node(master_gain)?;

            // Start the oscillator
            oscillator.start()?;

            // Store for later cleanup
            self.active_oscillators.insert(track_id, (oscillator, gain_node));
        }

        Ok(())
    }

    pub fn stop_note(&mut self, track_id: usize) -> Result<(), JsValue> {
        if let Some((oscillator, _gain_node)) = self.active_oscillators.remove(&track_id) {
            oscillator.stop()?;
        }
        Ok(())
    }

    pub fn stop_all_notes(&mut self) -> Result<(), JsValue> {
        for (_track_id, (oscillator, _gain_node)) in self.active_oscillators.drain() {
            oscillator.stop()?;
        }
        Ok(())
    }

    fn waveform_to_web_sys(&self, waveform: &WaveformType) -> Result<web_sys::OscillatorType, JsValue> {
        match waveform {
            WaveformType::Sine => Ok(web_sys::OscillatorType::Sine),
            WaveformType::Square => Ok(web_sys::OscillatorType::Square),
            WaveformType::Triangle => Ok(web_sys::OscillatorType::Triangle),
            WaveformType::Sawtooth => Ok(web_sys::OscillatorType::Sawtooth),
        }
    }
}