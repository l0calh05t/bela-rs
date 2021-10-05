use bela::{Bela, BelaApplication, Error, RenderContext};

struct HelloExample(usize);

// implementing BelaApplication is unsafe, as render may not
// perform any system calls (allocations, printing, etc.) and
// should never panic or block, which we cannot check at
// compile time
// TODO: what if panic = abort?
unsafe impl BelaApplication for HelloExample {
    fn render(&mut self, context: &mut RenderContext) {
        let audio_out_channels = context.audio_out_channels();
        for frame in context.audio_out().chunks_exact_mut(audio_out_channels) {
            let gain = 0.5;
            let signal = 2. * (self.0 as f32 * 110. / 44100.) - 1.;
            self.0 += 1;
            if self.0 as f32 > 44100. / 110. {
                self.0 = 0;
            }
            for sample in frame {
                *sample = gain * signal;
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
