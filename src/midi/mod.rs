use std::fs::File;
use std::io::Write;
use std::path::Path;
use midly::{Smf, Track, TrackEvent, MidiMessage, Header, Format, Timing};
use crate::state::{RecordedNote, State};
use crate::music_theory::note::Note;

pub mod export;
pub mod import;

/// Convert our Note enum to MIDI note number
pub fn note_to_midi_number(note: Note, octave: i32) -> u8 {
    let base_note = match note {
        Note::C => 0,
        Note::CSharp => 1,
        Note::D => 2,
        Note::DSharp => 3,
        Note::E => 4,
        Note::F => 5,
        Note::FSharp => 6,
        Note::G => 7,
        Note::GSharp => 8,
        Note::A => 9,
        Note::ASharp => 10,
        Note::B => 11,
    };
    
    // MIDI note 60 = C4
    ((octave + 1) * 12 + base_note as i32).clamp(0, 127) as u8
}

/// Convert MIDI note number back to our Note enum and octave
pub fn midi_number_to_note(midi_note: u8) -> (Note, i32) {
    let note_in_octave = midi_note % 12;
    let octave = (midi_note / 12) as i32 - 1;
    
    let note = match note_in_octave {
        0 => Note::C,
        1 => Note::CSharp,
        2 => Note::D,
        3 => Note::DSharp,
        4 => Note::E,
        5 => Note::F,
        6 => Note::FSharp,
        7 => Note::G,
        8 => Note::GSharp,
        9 => Note::A,
        10 => Note::ASharp,
        11 => Note::B,
        _ => Note::C, // Fallback
    };
    
    (note, octave)
}

/// Convert seconds to MIDI ticks (assuming 480 ticks per quarter note, 120 BPM)
pub fn seconds_to_ticks(seconds: f32) -> u32 {
    let ticks_per_quarter = 480.0;
    let bpm = 120.0;
    let quarters_per_second = bpm / 60.0;
    (seconds * quarters_per_second * ticks_per_quarter) as u32
}

/// Convert MIDI ticks back to seconds
pub fn ticks_to_seconds(ticks: u32) -> f32 {
    let ticks_per_quarter = 480.0;
    let bpm = 120.0;
    let quarters_per_second = bpm / 60.0;
    ticks as f32 / (quarters_per_second * ticks_per_quarter)
}