//! Safe Rust wrapper for the Bela platform API
//!
//! Wraps the [`bela-sys`](https://github.com/andrewcsmith/bela-sys) in
//! a safe API, simplifying the creation of Bela applications using safe
//! Rust. The main entry point is `Bela::new` followed by `Bela::run` as
//! shown in the example below:
//!
//! ```no_run
//! use bela::{Bela, BelaApplication, Error, RenderContext};
//!
//! struct Example(usize);
//!
//! // Implementing BelaApplication is unsafe, even when render only
//! // contains safe code, as it must also be realtime safe
//! unsafe impl BelaApplication for Example {
//!     fn render(&mut self, context: &mut RenderContext) {
//!         let audio_out_channels = context.audio_out_channels();
//!         for frame in context.audio_out().chunks_exact_mut(audio_out_channels) {
//!             let gain = 0.5;
//!             let signal = 2. * (self.0 as f32 * 110. / 44100.) - 1.;
//!             self.0 += 1;
//!             if self.0 as f32 > 44100. / 110. {
//!                 self.0 = 0;
//!             }
//!             for sample in frame {
//!                 *sample = gain * signal;
//!             }
//!         }
//!     }
//! }
//!
//! fn main() -> Result<(), Error> {
//!     Bela::new(|_context| Some(Example(0))).run()
//! }
//! ```

use nix::sys::signal;
use std::convert::TryInto;
use std::ffi::c_void;
use std::ops::DerefMut;
use std::os::raw::c_int;
use std::panic::{catch_unwind, UnwindSafe};
use std::thread::sleep;
use std::time::Duration;

mod init_settings;
use crate::init_settings::*;

mod context;
pub use crate::context::*;

mod bela_hw;
pub use crate::bela_hw::*;

mod error;
pub use crate::error::*;

mod auxiliary_task;
pub use crate::auxiliary_task::*;

#[cfg(feature = "midi")]
mod midi;
#[cfg(feature = "midi")]
pub use crate::midi::*;

/// Trait your Bela applications must implement
///
/// # Safety:
/// Implementing `BelaApplication` is unsafe, as some invariants
/// must be fulfilled that the Rust compiler cannot check. As
/// [per the official documentation](https://learn.bela.io/using-bela/languages/c-plus-plus/#bela-c-must-be-real-time-safe)
/// `render` must be real-time safe:
///
/// > The code in the `render()` function operates in “primary” mode.
/// > Everything that the operating system does is at a lower priority -
/// > such as connecting to the network, printing to the console,
/// > reading from the SD card, and so on - also called “secondary”
/// > mode.
/// >
/// > Switching from the primary real-time mode of the `render()` thread
/// > to the secondary system functions is called a mode switch, and
/// > will interfere with Bela’s performance.
/// >
/// > There are ways around mode switches. As a general rule, avoid
/// > running any code that requires access to specific features of the
/// > Linux system. For instance, instead of using the usual C++ methods
/// > of printing to the console such as `cout` or `printf()`, Bela has
/// > a real-time safe print function: `rt_printf()`. Additionally, when
/// > a process needs more time to complete or may involve switching
/// > modes, you can use the `AuxiliaryTask` API (see the example
/// > Sensors -> MPR121 in the Examples to run the code in a different,
/// > lower-priority thread.
/// >
/// > Be aware that this concept of real-time safety also applies to
/// > allocating memory, which should only be done in the `setup()`
/// > function and not in `render()`.
///
/// Additionally, `render` *must not* panic (a concept that does not
/// exist in C).
pub unsafe trait BelaApplication: Sized + Send {
    fn render(&mut self, context: &mut RenderContext);
}

/// The main entry point for Bela applications
///
/// `Bela` follows the builder pattern and allows you to incrementally
/// set up the Bela as required and then run it, consuming the builder
/// object. The most important functions are `Bela::new` and
/// `Bela::run`.
pub struct Bela<Constructor> {
    /// Initial settings. Not directly accessible.
    settings: InitSettings,
    /// `BelaApplication` constructor, set on creation.
    constructor: Constructor,
}

/// Internal user data definition
///
/// `UserData` can either be empty (`None`), contain a `BelaApplication`
/// constructor (`Constructor`) or a running application
/// (`Application`).
enum UserData<Application, Constructor> {
    /// A running `BelaApplication` application
    Application(Application),
    /// A `BelaApplication` constructor
    Constructor(Constructor),
    /// An empty `UserData` object
    None,
}

impl<Application, Constructor> UserData<Application, Constructor> {
    /// Take the `UserData` object and replace it with `None`, mirroring
    /// `Option::take`.
    fn take(&mut self) -> Self {
        std::mem::replace(self, Self::None)
    }

    /// Check if the `UserData` represents a running application.
    fn is_application(&self) -> bool {
        matches!(self, Self::Application(_))
    }
}

impl<Application, Constructor> Bela<Constructor>
where
    Application: BelaApplication,
    Constructor: Send + UnwindSafe + FnOnce(&mut Context<SetupTag>) -> Option<Application>,
{
    /// Create a new `Bela` builder with default settings from a
    /// `BelaApplication` constructor function object
    pub fn new(constructor: Constructor) -> Self {
        Self {
            settings: InitSettings::default(),
            constructor,
        }
    }

    /// Set number of analog frames per period (buffer). Number of audio frames
    /// depends on relative sample rates of the two. By default, audio is twice
    /// the sample rate, so has twice the period size.
    pub fn period_size(mut self, size: usize) -> Self {
        self.settings.periodSize = size.try_into().unwrap();
        self
    }

    /// Set whether to use the analog input and output
    pub fn use_analog(mut self, use_analog: bool) -> Self {
        self.settings.useAnalog = use_analog as _;
        self
    }

    /// Set whether to use the digital input and output
    pub fn use_digital(mut self, use_digital: bool) -> Self {
        self.settings.useDigital = use_digital as _;
        self
    }

    /// Set number of requested analog input channels
    pub fn num_analog_in_channels(mut self, num: usize) -> Self {
        self.settings.numAnalogInChannels = num.try_into().unwrap();
        self
    }

    /// Set number of requested analog output channels
    pub fn num_analog_out_channels(mut self, num: usize) -> Self {
        self.settings.numAnalogOutChannels = num.try_into().unwrap();
        self
    }

    /// Set number of requested digital channels
    pub fn num_digital_channels(mut self, num: usize) -> Self {
        self.settings.numDigitalChannels = num.try_into().unwrap();
        self
    }

    /// Set if Bela application should begin with speakers muted
    pub fn begin_muted(mut self, val: bool) -> Self {
        self.settings.beginMuted = val as _;
        self
    }

    /// Set initial audio DAC level
    pub fn dac_level(mut self, val: f32) -> Self {
        assert!(val.is_finite());
        self.settings.dacLevel = val;
        self
    }

    /// Set initial audio ADC level
    pub fn adc_level(mut self, val: f32) -> Self {
        assert!(val.is_finite());
        self.settings.adcLevel = val;
        self
    }

    /// Set initial gain for left and right PGA channels
    pub fn pga_gain(mut self, val: [f32; 2]) -> Self {
        assert!(val[0].is_finite());
        assert!(val[1].is_finite());
        self.settings.pgaGain = val;
        self
    }

    /// Set initial headphone level
    pub fn headphone_level(mut self, val: f32) -> Self {
        assert!(val.is_finite());
        self.settings.headphoneLevel = val;
        self
    }

    /// Set the number of requested multiplexer channels
    pub fn num_mux_channels(mut self, val: usize) -> Self {
        self.settings.numMuxChannels = val.try_into().unwrap();
        self
    }

    /// Set the number of requested audio expander inputs
    pub fn audio_expander_inputs(mut self, val: usize) -> Self {
        self.settings.audioExpanderInputs = val.try_into().unwrap();
        self
    }

    /// Set the number of requested audio expander outputs
    pub fn audio_expander_outputs(mut self, val: usize) -> Self {
        self.settings.audioExpanderOutputs = val.try_into().unwrap();
        self
    }

    /// Set the PRU (0 or 1) on which the code shall run
    ///
    /// # Safety
    /// TODO: unclear if this should be considered safe or not
    pub unsafe fn pru_number(mut self, val: i32) -> Self {
        assert!(val == 0 || val == 1);
        self.settings.pruNumber = val;
        self
    }

    /// The external .bin file to load. If empty will use PRU code from pru_rtaudio_bin.h
    ///
    /// # Safety
    /// The safety of the external PRU .bin file must be ascertained before calling this
    pub unsafe fn pru_filename(mut self, val: [u8; 256]) -> Self {
        self.settings.pruFilename = val;
        self
    }

    /// Enable or disable underrun detection and logging
    pub fn detect_underruns(mut self, val: bool) -> Self {
        self.settings.detectUnderruns = val as _;
        self
    }

    /// Enable or disable verbose logging
    pub fn verbose(mut self, val: bool) -> Self {
        self.settings.verbose = val as _;
        self
    }

    /// Enable or disable blinking LED indicating Bela is running
    pub fn enable_led(mut self, val: bool) -> Self {
        self.settings.enableLED = val as _;
        self
    }

    /// Set or disable the stop button pin (0-127)
    pub fn stop_button_pin(mut self, val: Option<i8>) -> Self {
        self.settings.stopButtonPin = match val {
            Some(v) => {
                assert!(v >= 0);
                v as _
            }
            None => -1,
        };
        self
    }

    /// Enable or disable high performance mode.
    /// May negatively affect IDE / Linux performance
    pub fn high_performance_mode(mut self, val: bool) -> Self {
        self.settings.highPerformanceMode = val as _;
        self
    }

    /// Enable or disable interleaving of audio samples
    pub fn interleave(mut self, val: bool) -> Self {
        self.settings.interleave = val as _;
        self
    }

    /// Set if analog outputs should persist
    pub fn analog_outputs_persist(mut self, val: bool) -> Self {
        self.settings.analogOutputsPersist = val as _;
        self
    }

    /// Set if analog inputs should be resampled to audio rate
    pub fn uniform_sample_rate(mut self, val: bool) -> Self {
        self.settings.uniformSampleRate = val as _;
        self
    }

    /// Set the requested audio thread stack size
    pub fn audio_thread_stack_size(mut self, num: usize) -> Self {
        self.settings.audioThreadStackSize = num.try_into().unwrap();
        self
    }

    /// Set the requested stack size for all auxiliary task threads
    pub fn auxiliary_task_stack_size(mut self, num: usize) -> Self {
        self.settings.auxiliaryTaskStackSize = num.try_into().unwrap();
        self
    }

    /// Set or disable the amplifier mute button pin (0-127)
    pub fn amp_mute_pin(mut self, val: Option<i8>) -> Self {
        self.settings.ampMutePin = match val {
            Some(v) => {
                assert!(v >= 0);
                v as _
            }
            None => -1,
        };
        self
    }

    /// Set user selected board to work with (as opposed to detected hardware)
    pub fn board(mut self, board: BelaHw) -> Self {
        self.settings.board = board as _;
        self
    }

    /// Consumes the `Bela` object and runs the application
    ///
    /// Terminates on error, or as soon as the application stops
    pub fn run(self) -> Result<(), Error> {
        let Self {
            mut settings,
            constructor,
        } = self;

        let mut user_data: UserData<Application, _> = UserData::Constructor(constructor);

        extern "C" fn setup_trampoline<Application, Constructor>(
            context: *mut bela_sys::BelaContext,
            user_data: *mut c_void,
        ) -> bool
        where
            Application: BelaApplication,
            Constructor: Send + UnwindSafe + FnOnce(&mut Context<SetupTag>) -> Option<Application>,
        {
            // create application instance
            // constructor is consumed
            let user_data = unsafe { &mut *(user_data as *mut UserData<Application, Constructor>) };
            let constructor = user_data.take();
            if let UserData::Constructor(constructor) = constructor {
                *user_data = match catch_unwind(|| {
                    let mut context = unsafe { Context::<SetupTag>::new(context) };
                    constructor(&mut context)
                }) {
                    Ok(application) => application.map_or(UserData::None, UserData::Application),
                    Err(_) => UserData::None,
                };
            }
            user_data.is_application()
        }

        extern "C" fn render_trampoline<Application, Constructor>(
            context: *mut bela_sys::BelaContext,
            user_data: *mut c_void,
        ) where
            Application: BelaApplication,
        {
            let user_data = unsafe { &mut *(user_data as *mut UserData<Application, Constructor>) };
            if let UserData::Application(user_data) = user_data {
                // NOTE: cannot use catch_unwind safely here, as it returns a boxed error (-> allocation in RT thread)
                let mut context = unsafe { Context::<RenderTag>::new(context) };
                user_data.render(&mut context);
            };
        }

        extern "C" fn cleanup_trampoline<Application, Constructor>(
            _context: *mut bela_sys::BelaContext,
            user_data: *mut c_void,
        ) {
            let _ = catch_unwind(|| {
                // drop application instance
                let user_data =
                    unsafe { &mut *(user_data as *mut UserData<Application, Constructor>) };
                user_data.take();
            });
        }

        settings.setup = Some(setup_trampoline::<Application, Constructor>);
        settings.render = Some(render_trampoline::<Application, Constructor>);
        settings.cleanup = Some(cleanup_trampoline::<Application, Constructor>);

        setup_signal_handler();

        if unsafe {
            bela_sys::Bela_initAudio(
                settings.deref_mut() as *mut _,
                &mut user_data as *mut _ as *mut _,
            )
        } != 0
        {
            return Err(Error::Init);
        }

        if unsafe { bela_sys::Bela_startAudio() } != 0 {
            return Err(Error::Start);
        }

        while unsafe { bela_sys::Bela_stopRequested() == 0 } {
            sleep(Duration::new(0, 10000));
        }

        unsafe {
            bela_sys::Bela_stopAudio();
            bela_sys::Bela_cleanupAudio();
        }

        Ok(())
    }
}

/// Internal function that setups up a Linux signal handler to stop the
/// Bela audio thread on `SIGINT` and `SIGTERM`
fn setup_signal_handler() {
    extern "C" fn signal_handler(_signal: c_int) {
        unsafe {
            bela_sys::Bela_requestStop();
        }
    }

    let handler = signal::SigHandler::Handler(signal_handler);
    let flags = signal::SaFlags::empty();
    let set = signal::SigSet::empty();
    let sig_action = signal::SigAction::new(handler, flags, set);

    unsafe {
        signal::sigaction(signal::SIGINT, &sig_action).unwrap();
        signal::sigaction(signal::SIGTERM, &sig_action).unwrap();
    }
}
