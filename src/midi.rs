#[derive(Debug)]
pub struct Midi(*mut bela_sys::midi::Midi);

impl Midi {
    pub fn new(port: &std::ffi::CStr) -> Option<Self> {
        let midi = unsafe { bela_sys::midi::Midi_new(port.as_ptr()) };
        if midi.is_null() {
            None
        } else {
            Some(Midi(midi))
        }
    }

    pub fn get_message<'buf>(&mut self, buf: &'buf mut [u8; 3]) -> Option<&'buf [u8]> {
        unsafe {
            if bela_sys::midi::Midi_availableMessages(self.0) <= 0 {
                None
            } else {
                let len = bela_sys::midi::Midi_getMessage(self.0, buf.as_mut_ptr()) as usize;
                debug_assert!(len > 0);
                debug_assert!(len <= 3);
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
