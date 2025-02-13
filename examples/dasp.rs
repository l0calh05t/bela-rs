use bela::{Bela, BelaApplication, BelaHw, Error, RenderContext};
use dasp::{signal, slice::to_frame_slice_mut, Signal};

struct DaspExample(Box<dyn Signal<Frame = f64> + Send>);

unsafe impl BelaApplication for DaspExample {
    fn render(&mut self, context: &mut RenderContext) {
        let audio_out = context.audio_out();
        let audio_out_frames: &mut [[f32; 2]] = to_frame_slice_mut(audio_out).unwrap();

        for frame in audio_out_frames.iter_mut() {
            for sample in frame.iter_mut() {
                let val = self.0.next();
                *sample = val as f32;
            }
        }
    }
}

fn main() -> Result<(), Error> {
    Bela::new(|ctx| {
        assert_eq!(ctx.audio_out_channels(), 2);
        let sig = signal::rate(ctx.audio_sample_rate() as f64)
            .const_hz(440.0)
            .sine();
        Some(DaspExample(Box::new(sig)))
    })
    .verbose(true)
    .board(BelaHw::Bela)
    .run()
}
