use crate::{Error, RenderContext, SetupContext};

pub struct Midi(*mut bela_sys::midi::Midi);

impl Drop for Midi {
    fn drop(&mut self) {
        unsafe {
            bela_sys::midi::Midi_delete(self.0);
        }
    }
}

unsafe impl Send for Midi {}

impl SetupContext {
    pub fn new_midi(&mut self, port: &std::ffi::CStr) -> Result<Midi, Error> {
        let midi = unsafe { bela_sys::midi::Midi_new(port.as_ptr()) };
        if midi.is_null() {
            Err(Error::Midi)
        } else {
            Ok(Midi(midi))
        }
    }
}

impl RenderContext {
    pub fn get_midi_message<'buffer>(
        &mut self,
        midi: &mut Midi,
        buffer: &'buffer mut [u8; 3],
    ) -> Option<&'buffer [u8]> {
        unsafe {
            if bela_sys::midi::Midi_availableMessages(midi.0) <= 0 {
                None
            } else {
                let len = bela_sys::midi::Midi_getMessage(midi.0, buffer.as_mut_ptr()) as usize;
                Some(&buffer[0..len])
            }
        }
    }
}
