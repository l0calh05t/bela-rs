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

pub unsafe trait BelaApplication: Sized + Send {
    fn render(&mut self, context: &mut Context<RenderTag>);
}

pub struct Bela<Constructor> {
    settings: InitSettings,
    constructor: Constructor,
}

enum UserData<Application, Constructor> {
    Application(Application),
    Constructor(Constructor),
    None,
}

impl<Application, Constructor> UserData<Application, Constructor> {
    fn take(&mut self) -> Self {
        std::mem::replace(self, Self::None)
    }

    fn is_application(&self) -> bool {
        matches!(self, Self::Application(_))
    }
}

impl<Application, Constructor> Bela<Constructor>
where
    Application: BelaApplication,
    Constructor: Send + UnwindSafe + FnOnce(&mut Context<SetupTag>) -> Option<Application>,
{
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

    /// Set user selected board to work with (as opposed to detected hardware).
    pub fn board(mut self, board: BelaHw) -> Self {
        self.settings.board = board as _;
        self
    }

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
