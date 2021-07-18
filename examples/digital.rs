use bela::{Bela, BelaApplication, DigitalDirection, Error, RenderContext};

struct DigitalExample(usize);

unsafe impl BelaApplication for DigitalExample {
    fn render(&mut self, context: &mut RenderContext) {
        context.pin_mode(0, 0, DigitalDirection::Output);
        let ten_ms_in_frames = (context.digital_sample_rate() / 100.) as usize;
        let hundred_ms_in_frames = (context.digital_sample_rate() / 10.) as usize;
        for f in 0..context.digital_frames() {
            let v = self.0 < ten_ms_in_frames;
            context.digital_write_once(f, 0, v);
            self.0 += 1;
            if self.0 > hundred_ms_in_frames {
                self.0 = 0;
            }
        }
    }
}

fn main() -> Result<(), Error> {
    Bela::new(|_| Some(DigitalExample(0))).run()
}
