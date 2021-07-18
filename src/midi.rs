use crate::{Error, RenderContext, SetupContext};

pub struct Midi(*mut bela_sys::midi::Midi);

impl Midi {
    pub fn new(port: &std::ffi::CStr, _context: &mut SetupContext) -> Result<Self, Error> {
        let midi = unsafe { bela_sys::midi::Midi_new(port.as_ptr()) };
        if midi.is_null() {
            Err(Error::Midi)
        } else {
            Ok(Midi(midi))
        }
    }

    pub fn get_message<'buf>(
        &mut self,
        buf: &'buf mut [u8; 3],
        _context: &mut RenderContext,
    ) -> Option<&'buf [u8]> {
        unsafe {
            if bela_sys::midi::Midi_availableMessages(self.0) <= 0 {
                None
            } else {
                let len = bela_sys::midi::Midi_getMessage(self.0, buf.as_mut_ptr()) as usize;
                Some(&buf[0..len])
            }
        }
    }
}

impl Drop for Midi {
    fn drop(&mut self) {
        unsafe {
            bela_sys::midi::Midi_delete(self.0);
        }
    }
}

unsafe impl Send for Midi {}
