use bela::{Bela, BelaApplication, Error, RenderContext};

struct HelloExample(usize);

// implementing BelaApplication is unsafe, as render may not
// perform any system calls (allocations, printing, etc.) and
// should never panic or block, which we cannot check at
// compile time
// TODO: what if panic = abort?
unsafe impl BelaApplication for HelloExample {
    fn render(&mut self, context: &mut RenderContext) {
        for sample in context.audio_out().iter_mut() {
            let gain = 0.5;
            *sample = 2. * (self.0 as f32 * 110. / 44100.) - 1.;
            *sample *= gain;
            self.0 += 1;
            if self.0 as f32 > 44100. / 110. {
                self.0 = 0;
            }
        }
    }
}

fn main() -> Result<(), Error> {
    Bela::new(|_| Some(HelloExample(0)))
        .verbose(true)
        .dac_level(-6.0)
        .high_performance_mode(true)
        .run()
}
