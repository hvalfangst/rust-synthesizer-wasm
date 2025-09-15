use std::fs::File;
use std::io::Read;
use crate::state::{RecordedNote, State};
use super::{midi_number_to_note, ticks_to_seconds};

/// Import MIDI file and convert to RecordedNote format using raw MIDI parsing
pub fn import_midi_to_track(file_path: &str) -> Result<Vec<RecordedNote>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    if buffer.len() < 14 {
        return Err("Invalid MIDI file: too short".into());
    }
    
    // Check MIDI header
    if &buffer[0..4] != b"MThd" {
        return Err("Invalid MIDI file: missing header".into());
    }
    
    let header_length = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    if header_length != 6 {
        return Err("Invalid MIDI file: wrong header length".into());
    }
    
    let format = u16::from_be_bytes([buffer[8], buffer[9]]);
    let num_tracks = u16::from_be_bytes([buffer[10], buffer[11]]);
    let ticks_per_quarter = u16::from_be_bytes([buffer[12], buffer[13]]);
    
    let mut recorded_notes = Vec::new();
    let mut pos = 14; // Start after header
    
    // Process each track
    for _ in 0..num_tracks {
        if pos + 8 > buffer.len() {
            break;
        }
        
        // Check track header
        if &buffer[pos..pos+4] != b"MTrk" {
            return Err("Invalid MIDI file: missing track header".into());
        }
        
        let track_length = u32::from_be_bytes([
            buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]
        ]) as usize;
        
        pos += 8; // Skip track header
        let track_end = pos + track_length;
        
        if track_end > buffer.len() {
            break;
        }
        
        // Parse track events
        let track_notes = parse_track_events(&buffer[pos..track_end], ticks_per_quarter)?;
        recorded_notes.extend(track_notes);
        
        pos = track_end;
    }
    
    // Sort by timestamp
    recorded_notes.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
    
    println!("Imported {} notes from MIDI file: {}", recorded_notes.len(), file_path);
    Ok(recorded_notes)
}

/// Parse track events from raw MIDI data
fn parse_track_events(data: &[u8], ticks_per_quarter: u16) -> Result<Vec<RecordedNote>, Box<dyn std::error::Error>> {
    let mut notes = Vec::new();
    let mut pos = 0;
    let mut current_ticks = 0u32;
    let mut active_notes: std::collections::HashMap<u8, f32> = std::collections::HashMap::new();
    let mut running_status = 0u8;
    
    while pos < data.len() {
        // Read variable length delta time
        let (delta_time, delta_bytes) = read_variable_length(&data[pos..])?;
        pos += delta_bytes;
        current_ticks += delta_time;
        
        if pos >= data.len() {
            break;
        }
        
        let mut status = data[pos];
        
        // Handle running status
        if status < 0x80 {
            status = running_status;
        } else {
            pos += 1;
            running_status = status;
        }
        
        match status & 0xF0 {
            0x80 => { // Note Off
                if pos + 1 >= data.len() { break; }
                let note = data[pos];
                let _velocity = data[pos + 1];
                pos += 2;
                
                let current_time = ticks_to_seconds_custom(current_ticks, ticks_per_quarter);
                if let Some(start_time) = active_notes.remove(&note) {
                    let duration = current_time - start_time;
                    let (note_enum, octave) = midi_number_to_note(note);
                    
                    notes.push(RecordedNote {
                        note: note_enum,
                        octave,
                        timestamp: start_time,
                        duration,
                    });
                }
            },
            0x90 => { // Note On
                if pos + 1 >= data.len() { break; }
                let note = data[pos];
                let velocity = data[pos + 1];
                pos += 2;
                
                let current_time = ticks_to_seconds_custom(current_ticks, ticks_per_quarter);
                
                if velocity == 0 {
                    // Velocity 0 is actually a note off
                    if let Some(start_time) = active_notes.remove(&note) {
                        let duration = current_time - start_time;
                        let (note_enum, octave) = midi_number_to_note(note);
                        
                        notes.push(RecordedNote {
                            note: note_enum,
                            octave,
                            timestamp: start_time,
                            duration,
                        });
                    }
                } else {
                    active_notes.insert(note, current_time);
                }
            },
            0xA0 => { // Polyphonic Pressure
                if pos + 1 >= data.len() { break; }
                pos += 2; // Skip
            },
            0xB0 => { // Control Change
                if pos + 1 >= data.len() { break; }
                pos += 2; // Skip
            },
            0xC0 => { // Program Change
                if pos >= data.len() { break; }
                pos += 1; // Skip
            },
            0xD0 => { // Channel Pressure
                if pos >= data.len() { break; }
                pos += 1; // Skip
            },
            0xE0 => { // Pitch Bend
                if pos + 1 >= data.len() { break; }
                pos += 2; // Skip
            },
            0xF0 => { // System messages
                if status == 0xFF { // Meta event
                    if pos + 1 >= data.len() { break; }
                    let meta_type = data[pos];
                    pos += 1;
                    
                    let (length, length_bytes) = read_variable_length(&data[pos..])?;
                    pos += length_bytes + length as usize;
                } else {
                    // Other system messages - skip
                    pos += 1;
                }
            },
            _ => {
                // Unknown event, try to skip
                pos += 1;
            }
        }
    }
    
    // Handle any remaining active notes
    let final_time = ticks_to_seconds_custom(current_ticks, ticks_per_quarter);
    for (note, start_time) in active_notes {
        let duration = final_time - start_time;
        let (note_enum, octave) = midi_number_to_note(note);
        
        notes.push(RecordedNote {
            note: note_enum,
            octave,
            timestamp: start_time,
            duration,
        });
    }
    
    Ok(notes)
}

/// Read variable length quantity from MIDI data
fn read_variable_length(data: &[u8]) -> Result<(u32, usize), Box<dyn std::error::Error>> {
    let mut value = 0u32;
    let mut bytes_read = 0;
    
    for &byte in data.iter().take(4) {
        value = (value << 7) | ((byte & 0x7F) as u32);
        bytes_read += 1;
        
        if byte & 0x80 == 0 {
            break;
        }
    }
    
    if bytes_read == 0 {
        return Err("Invalid variable length quantity".into());
    }
    
    Ok((value, bytes_read))
}

/// Convert MIDI ticks to seconds with custom ticks per quarter
fn ticks_to_seconds_custom(ticks: u32, ticks_per_quarter: u16) -> f32 {
    let bpm = 120.0;
    let quarters_per_second = bpm / 60.0;
    ticks as f32 / (quarters_per_second * ticks_per_quarter as f32)
}

/// Import MIDI file to a specific track in the synthesizer state
pub fn import_midi_to_synthesizer_track(state: &mut State, track_id: usize, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if track_id >= state.tracks.len() {
        return Err("Invalid track ID".into());
    }
    
    let recorded_notes = import_midi_to_track(file_path)?;
    state.tracks[track_id].recorded_notes = recorded_notes;
    
    println!("MIDI imported to track {}: {}", track_id + 1, state.tracks[track_id].name);
    Ok(())
}