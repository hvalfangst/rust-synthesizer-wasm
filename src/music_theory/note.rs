use std::fmt;

/// Enumerates musical notes C4 through B5
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Note {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B
}

/// Implements the [Display] trait for [Note]
impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Note::A => write!(f, "A"),
            Note::ASharp => write!(f, "A#"),
            Note::B => write!(f, "B"),
            Note::C => write!(f, "C"),
            Note::CSharp => write!(f, "C#"),
            Note::D => write!(f, "D"),
            Note::DSharp => write!(f, "D#"),
            Note::E => write!(f, "E"),
            Note::F => write!(f, "F"),
            Note::FSharp => write!(f, "F#"),
            Note::G => write!(f, "G"),
            _ => write!(f, "G#")
        }
    }
}

impl Note {
    /// Computes the frequency of the note.rs based on the following: [frequency * (2^(octave-4))].
    ///
    /// # Arguments
    ///
    /// * `octave` - The current octave.
    ///
    /// # Returns
    ///
    /// The adjusted frequency of the note.rs based on the current octave.
    pub fn frequency(&self, octave: i32) -> f32 {
        let base_frequency = match self {
            Note::C => 261.63,
            Note::CSharp => 277.18,
            Note::D => 293.66,
            Note::DSharp => 311.13,
            Note::E => 329.63,
            Note::F => 349.23,
            Note::FSharp => 369.99,
            Note::G => 392.00,
            Note::GSharp => 415.30,
            Note::A => 440.0,
            Note::ASharp => 466.16,
            Note::B => 493.88,
        };

        // Adjust the base frequency based on the current octave setting
        base_frequency * 2.0_f32.powi(octave - 4)
    }

    /// Create a Note from a string representation
    pub fn from_str(s: &str) -> Result<Note, &'static str> {
        match s {
            "C" => Ok(Note::C),
            "C#" => Ok(Note::CSharp),
            "D" => Ok(Note::D),
            "D#" => Ok(Note::DSharp),
            "E" => Ok(Note::E),
            "F" => Ok(Note::F),
            "F#" => Ok(Note::FSharp),
            "G" => Ok(Note::G),
            "G#" => Ok(Note::GSharp),
            "A" => Ok(Note::A),
            "A#" => Ok(Note::ASharp),
            "B" => Ok(Note::B),
            _ => Err("Invalid note name"),
        }
    }
}