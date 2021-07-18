use bela::midi::Midi;
use bela::{Bela, BelaApplication, Error, RenderContext, SetupContext};

struct MidiExample(Midi);

impl MidiExample {
    fn new(context: &mut SetupContext) -> Option<MidiExample> {
        Midi::new(
            std::ffi::CStr::from_bytes_with_nul(b"hw:0,0,0\0").unwrap(),
            context,
        )
        .map(MidiExample)
        .ok()
    }
}

unsafe impl BelaApplication for MidiExample {
    fn render(&mut self, context: &mut RenderContext) {
        let mut buf = [0u8; 3];
        while let Some(msg) = self.0.get_message(&mut buf, context) {
            if (msg[0] & 0b1111_0000) == 0b1001_0000 {
                unsafe { bela_sys::rt_printf(b"Note on\n\0".as_ptr()) };
            }
        }
    }
}

fn main() -> Result<(), Error> {
    Bela::new(MidiExample::new).verbose(true).run()
}
