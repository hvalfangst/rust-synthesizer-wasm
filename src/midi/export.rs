use std::fs::File;
use std::io::Write;
use midly::{Smf, Header, Format, Timing, Track, TrackEvent, TrackEventKind, MidiMessage, MetaMessage};
use crate::state::{RecordedNote, State};
use crate::music_theory::note::Note;
use super::{note_to_midi_number, seconds_to_ticks};

/// Export a single track to MIDI
pub fn export_track_to_midi(track_notes: &[RecordedNote], track_name: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create MIDI header
    let header = Header {
        format: Format::SingleTrack,
        timing: Timing::Metrical(480.into()),
    };
    
    // Create track events
    let mut events = Vec::new();
    
    // Add track name
    events.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(track_name.as_bytes())),
    });
    
    // Sort notes by timestamp for proper MIDI timing
    let mut sorted_notes = track_notes.to_vec();
    sorted_notes.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
    
    let mut last_time_ticks = 0u32;
    
    for recorded_note in &sorted_notes {
        let note_on_time = seconds_to_ticks(recorded_note.timestamp);
        let note_off_time = seconds_to_ticks(recorded_note.timestamp + recorded_note.duration);
        let midi_note = note_to_midi_number(recorded_note.note, recorded_note.octave);
        
        // Note On event
        let delta_on = note_on_time.saturating_sub(last_time_ticks);
        events.push(TrackEvent {
            delta: delta_on.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: MidiMessage::NoteOn {
                    key: midi_note.into(),
                    vel: 100.into(), // Fixed velocity for now
                },
            },
        });
        
        // Note Off event
        let delta_off = note_off_time.saturating_sub(note_on_time);
        events.push(TrackEvent {
            delta: delta_off.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: MidiMessage::NoteOff {
                    key: midi_note.into(),
                    vel: 0.into(),
                },
            },
        });
        
        last_time_ticks = note_off_time;
    }
    
    // End of track
    events.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });
    
    // Create SMF and save to file
    let track: Track = events;
    let smf = Smf {
        header,
        tracks: vec![track],
    };
    
    // Write to buffer first, then to file
    let mut buffer = Vec::new();
    smf.write(&mut buffer)?;
    
    let mut file = File::create(file_path)?;
    file.write_all(&buffer)?;
    
    println!("MIDI file exported: {}", file_path);
    Ok(())
}

/// Export all tracks from the synthesizer state to separate MIDI files
pub fn export_all_tracks_to_midi(state: &State, base_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    for (i, track) in state.tracks.iter().enumerate() {
        if !track.recorded_notes.is_empty() {
            let file_path = format!("{}_{}.mid", base_path, track.name);
            export_track_to_midi(&track.recorded_notes, &track.name, &file_path)?;
        }
    }
    Ok(())
}

/// Export all tracks to a single multi-track MIDI file
pub fn export_multitrack_midi(state: &State, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create MIDI header (Type 1 = multi-track)
    let track_count = state.tracks.iter().filter(|t| !t.recorded_notes.is_empty()).count() as u16;
    let header = Header {
        format: Format::Parallel,
        timing: Timing::Metrical(480.into()),
    };
    
    let mut tracks = Vec::new();
    
    for track in &state.tracks {
        if track.recorded_notes.is_empty() {
            continue;
        }
        
        let mut events = Vec::new();
        
        // Add track name
        events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::TrackName(track.name.as_bytes())),
        });
        
        // Sort notes by timestamp
        let mut sorted_notes = track.recorded_notes.clone();
        sorted_notes.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
        
        let mut last_time_ticks = 0u32;
        
        for recorded_note in &sorted_notes {
            let note_on_time = seconds_to_ticks(recorded_note.timestamp);
            let note_off_time = seconds_to_ticks(recorded_note.timestamp + recorded_note.duration);
            let midi_note = note_to_midi_number(recorded_note.note, recorded_note.octave);
            
            // Note On event
            let delta_on = note_on_time.saturating_sub(last_time_ticks);
            events.push(TrackEvent {
                delta: delta_on.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn {
                        key: midi_note.into(),
                        vel: 100.into(),
                    },
                },
            });
            
            // Note Off event
            let delta_off = note_off_time.saturating_sub(note_on_time);
            events.push(TrackEvent {
                delta: delta_off.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOff {
                        key: midi_note.into(),
                        vel: 0.into(),
                    },
                },
            });
            
            last_time_ticks = note_off_time;
        }
        
        // End of track
        events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
        
        tracks.push(events);
    }
    
    // Create SMF and save to file
    let smf = Smf {
        header,
        tracks,
    };
    
    // Write to buffer first, then to file
    let mut buffer = Vec::new();
    smf.write(&mut buffer)?;
    
    let mut file = File::create(file_path)?;
    file.write_all(&buffer)?;
    
    println!("Multi-track MIDI file exported: {}", file_path);
    Ok(())
}