extern crate bela;

use bela::midi::*;
use bela::*;

fn main() {
    go().unwrap();
}

fn go() -> Result<(), error::Error> {
    let mut setup =
        |_context: &mut Context, user_data: &mut Option<Midi>| -> Result<(), error::Error> {
            *user_data = Midi::new(&std::ffi::CStr::from_bytes_with_nul(b"hw:0,0,0\0").unwrap());
            Ok(())
        };

    let mut cleanup = |_context: &mut Context, user_data: &mut Option<Midi>| {
        *user_data = None;
    };

    let mut render = |_context: &mut Context, user_data: &mut Option<Midi>| {
        if let Some(midi) = user_data {
            let mut buf = [0u8; 3];
            while let Some(msg) = midi.get_message(&mut buf) {
                if (msg[0] & 0b1111_0000) == 0b1001_0000 {
                    unsafe { bela_sys::rt_printf(b"Note on\n\0".as_ptr()) };
                }
            }
        }
    };

    let midi: Option<Midi> = None;
    let user_data = AppData::new(midi, &mut render, Some(&mut setup), Some(&mut cleanup));

    let mut bela_app = Bela::new(user_data);
    let mut settings = InitSettings::default();
    bela_app.run(&mut settings)
}
