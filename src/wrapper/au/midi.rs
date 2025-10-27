//! Audio Unit MIDI event handling.
//!
//! This module implements MIDI event processing for Audio Unit plugins that support
//! MIDI input (MusicDevice and MusicEffect component types).

use std::ffi::c_void;

use crate::midi::NoteEvent;
use crate::plugin::Plugin;

use super::bindings::{MIDIEvent, MIDIEventList};
use super::factory::AudioComponentPlugInInstance;

impl<P: Plugin> AudioComponentPlugInInstance<P> {
    /// Handle MIDI input events from the AU host.
    ///
    /// This is called by the AU host when MIDI events are received for a MusicDevice
    /// or MusicEffect plugin. The events are converted to NIH-plug format and added
    /// to the plugin's MIDI event queue.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers safely.
    pub unsafe extern "C" fn au_midi_input(
        instance: *mut c_void,
        _timestamp: u64,
        _midi_list: *const MIDIEventList,
    ) -> i32 {
        if instance.is_null() {
            return super::bindings::errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);

        // For now, we'll implement a basic MIDI event processing
        // In a full implementation, we would:
        // 1. Parse the MIDI event list
        // 2. Convert AU MIDI events to NIH-plug NoteEvent format
        // 3. Add events to the wrapper's MIDI event queue

        nih_log!("AU MIDI input received (placeholder implementation)");

        // TODO: Implement actual MIDI event processing
        // This would involve:
        // 1. Iterating through the MIDI event list
        // 2. Converting each MIDI event to NoteEvent format
        // 3. Adding events to the wrapper's MIDI event queue

        super::bindings::errors::NO_ERR
    }

    /// Handle note start events.
    ///
    /// This is called by the AU host when a note should start playing.
    /// This is typically used for MusicDevice plugins.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers safely.
    pub unsafe extern "C" fn au_start_note(
        instance: *mut c_void,
        _note_number: u8,
        _velocity: u8,
        _channel: u8,
    ) -> i32 {
        if instance.is_null() {
            return super::bindings::errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);

        nih_log!("AU Start note received (placeholder implementation)");

        // TODO: Implement note start processing
        // This would involve:
        // 1. Creating a NoteEvent for note on
        // 2. Adding it to the wrapper's MIDI event queue

        super::bindings::errors::NO_ERR
    }

    /// Handle note stop events.
    ///
    /// This is called by the AU host when a note should stop playing.
    /// This is typically used for MusicDevice plugins.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers safely.
    pub unsafe extern "C" fn au_stop_note(
        instance: *mut c_void,
        _note_number: u8,
        _channel: u8,
    ) -> i32 {
        if instance.is_null() {
            return super::bindings::errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);

        nih_log!("AU Stop note received (placeholder implementation)");

        // TODO: Implement note stop processing
        // This would involve:
        // 1. Creating a NoteEvent for note off
        // 2. Adding it to the wrapper's MIDI event queue

        super::bindings::errors::NO_ERR
    }

    /// Convert an AU MIDI event to a NIH-plug NoteEvent.
    ///
    /// This helper function converts the AU MIDI event format to the NIH-plug
    /// NoteEvent format for processing by the plugin.
    fn convert_midi_event(au_event: &MIDIEvent) -> Option<NoteEvent<P::SysExMessage>> {
        // Extract MIDI data
        let status = au_event.status;
        let data1 = au_event.data1;
        let data2 = au_event.data2;
        let sample_offset = au_event.sample_offset;

        // Determine the MIDI message type
        let message_type = status & 0xF0;
        let channel = status & 0x0F;

        match message_type {
            0x80 => {
                // Note Off
                Some(NoteEvent::NoteOff {
                    timing: sample_offset as u32,
                    channel: channel as u8,
                    note: data1,
                    velocity: data2,
                })
            }
            0x90 => {
                // Note On (velocity 0 is treated as Note Off)
                if data2 == 0 {
                    Some(NoteEvent::NoteOff {
                        timing: sample_offset as u32,
                        channel: channel as u8,
                        note: data1,
                        velocity: 64, // Default velocity for note off
                    })
                } else {
                    Some(NoteEvent::NoteOn {
                        timing: sample_offset as u32,
                        channel: channel as u8,
                        note: data1,
                        velocity: data2,
                    })
                }
            }
            0xA0 => {
                // Polyphonic Key Pressure
                Some(NoteEvent::PolyPressure {
                    timing: sample_offset as u32,
                    channel: channel as u8,
                    note: data1,
                    pressure: data2,
                })
            }
            0xB0 => {
                // Control Change
                Some(NoteEvent::CC {
                    timing: sample_offset as u32,
                    channel: channel as u8,
                    cc: data1,
                    value: data2,
                })
            }
            0xC0 => {
                // Program Change
                Some(NoteEvent::ProgramChange {
                    timing: sample_offset as u32,
                    channel: channel as u8,
                    program: data1,
                })
            }
            0xD0 => {
                // Channel Pressure
                Some(NoteEvent::ChannelPressure {
                    timing: sample_offset as u32,
                    channel: channel as u8,
                    pressure: data1,
                })
            }
            0xE0 => {
                // Pitch Bend
                let pitch_bend = ((data2 as u16) << 7) | (data1 as u16);
                Some(NoteEvent::PitchBend {
                    timing: sample_offset as u32,
                    channel: channel as u8,
                    value: pitch_bend,
                })
            }
            _ => {
                // Unknown or unsupported MIDI message
                nih_log!("AU MIDI: Unsupported message type 0x{:02X}", message_type);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test plugin for testing MIDI conversion
    struct TestPlugin;
    impl crate::plugin::Plugin for TestPlugin {
        const NAME: &'static str = "Test Plugin";
        const VENDOR: &'static str = "Test Vendor";
        const VERSION: &'static str = "1.0.0";
        const AUDIO_IO_LAYOUTS: &'static [crate::prelude::AudioIOLayout] = &[];
        const MIDI_INPUT: crate::prelude::MidiConfig = crate::prelude::MidiConfig::None;
        const SAMPLE_ACCURATE_AUTOMATION: bool = false;
        const HARD_REALTIME_ONLY: bool = false;
        const SEND_FEEDBACK: bool = false;
        type SysExMessage = ();
        type BackgroundTask = ();
        fn params(&self) -> &dyn crate::prelude::Params { unimplemented!() }
        fn process(&mut self, _buffer: &mut crate::buffer::Buffer, _aux: &mut crate::prelude::AuxiliaryBuffers, _context: &mut impl crate::prelude::ProcessContext) -> crate::plugin::ProcessStatus { crate::plugin::ProcessStatus::Normal }
        fn deactivate(&mut self) {}
        fn reset(&mut self) {}
        fn initialize(&mut self, _audio_io_layout: &crate::prelude::AudioIOLayout, _buffer_config: &crate::prelude::BufferConfig, _context: &mut impl crate::prelude::InitContext) -> bool { true }
    }
    impl Default for TestPlugin { fn default() -> Self { Self } }

    #[test]
    fn test_midi_event_conversion_note_on() {
        let au_event = MIDIEvent {
            status: 0x90, // Note On, channel 0
            data1: 60,    // Middle C
            data2: 100,   // Velocity
            reserved: 0,
            sample_offset: 512,
        };

        let note_event = AudioComponentPlugInInstance::<TestPlugin>::convert_midi_event(&au_event);
        
        match note_event {
            Some(NoteEvent::NoteOn { timing, channel, note, velocity }) => {
                assert_eq!(timing, 512);
                assert_eq!(channel, 0);
                assert_eq!(note, 60);
                assert_eq!(velocity, 100);
            }
            _ => panic!("Expected NoteOn event"),
        }
    }

    #[test]
    fn test_midi_event_conversion_note_off() {
        let au_event = MIDIEvent {
            status: 0x80, // Note Off, channel 0
            data1: 60,    // Middle C
            data2: 64,    // Velocity
            reserved: 0,
            sample_offset: 1024,
        };

        let note_event = AudioComponentPlugInInstance::<TestPlugin>::convert_midi_event(&au_event);
        
        match note_event {
            Some(NoteEvent::NoteOff { timing, channel, note, velocity }) => {
                assert_eq!(timing, 1024);
                assert_eq!(channel, 0);
                assert_eq!(note, 60);
                assert_eq!(velocity, 64);
            }
            _ => panic!("Expected NoteOff event"),
        }
    }

    #[test]
    fn test_midi_event_conversion_cc() {
        let au_event = MIDIEvent {
            status: 0xB0, // Control Change, channel 0
            data1: 1,     // Modulation wheel
            data2: 127,   // Value
            reserved: 0,
            sample_offset: 256,
        };

        let note_event = AudioComponentPlugInInstance::<TestPlugin>::convert_midi_event(&au_event);
        
        match note_event {
            Some(NoteEvent::CC { timing, channel, cc, value }) => {
                assert_eq!(timing, 256);
                assert_eq!(channel, 0);
                assert_eq!(cc, 1);
                assert_eq!(value, 127);
            }
            _ => panic!("Expected CC event"),
        }
    }

    #[test]
    fn test_midi_event_conversion_pitch_bend() {
        let au_event = MIDIEvent {
            status: 0xE0, // Pitch Bend, channel 0
            data1: 0,     // LSB
            data2: 64,    // MSB (center position)
            reserved: 0,
            sample_offset: 128,
        };

        let note_event = AudioComponentPlugInInstance::<TestPlugin>::convert_midi_event(&au_event);
        
        match note_event {
            Some(NoteEvent::PitchBend { timing, channel, value }) => {
                assert_eq!(timing, 128);
                assert_eq!(channel, 0);
                assert_eq!(value, 8192); // (64 << 7) | 0
            }
            _ => panic!("Expected PitchBend event"),
        }
    }

    #[test]
    fn test_midi_event_conversion_unsupported() {
        let au_event = MIDIEvent {
            status: 0xF0, // System Exclusive (unsupported)
            data1: 0,
            data2: 0,
            reserved: 0,
            sample_offset: 0,
        };

        let note_event = AudioComponentPlugInInstance::<TestPlugin>::convert_midi_event(&au_event);
        assert!(note_event.is_none());
    }
}