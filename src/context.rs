use std::marker::PhantomData;
use std::slice::{from_raw_parts, from_raw_parts_mut};

/// Tag type to mark Bela contexts within `setup`
pub struct SetupTag;
/// Tag type to mark Bela contexts within `render`
pub struct RenderTag;

/// Bela context passed to setup/render/cleanup-functions.
/// `StateTag` represents the current state of the Bela application
pub struct Context<StateTag>(*mut bela_sys::BelaContext, PhantomData<StateTag>);

/// Type alias for a Bela context within `setup`
pub type SetupContext = Context<SetupTag>;
/// Type alias for a Bela context within `render`
pub type RenderContext = Context<RenderTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitalDirection {
    Input,
    Output,
}

// functions for all contexts (setup or render)
impl<StateTag> Context<StateTag> {
    /// Create a safe `Context` object from a raw `*mut bela_sys::BelaContext`
    ///
    /// # Safety
    /// The correct type state tag must be passed, i.e., `SetupTag` within
    /// `setup` and `RenderTag` within `render`. Furthermore, this object should
    /// not escape the confines of the containing `setup`/`render` and should
    /// therefore only be passed by mutable reference to user code
    pub(crate) unsafe fn new(context: *mut bela_sys::BelaContext) -> Self {
        Self(context, PhantomData)
    }

    /// Mutable reference to wrapped C struct
    // pub(crate) fn raw_mut(&mut self) -> &mut bela_sys::BelaContext {
    //     unsafe { &mut *self.0 }
    // }

    /// Reference to wrapped C struct
    pub(crate) fn raw(&self) -> &bela_sys::BelaContext {
        unsafe { &*self.0 }
    }

    pub fn audio_frames(&self) -> usize {
        self.raw().audioFrames as _
    }

    pub fn audio_in_channels(&self) -> usize {
        self.raw().audioInChannels as _
    }

    pub fn audio_out_channels(&self) -> usize {
        self.raw().audioOutChannels as _
    }

    pub fn audio_sample_rate(&self) -> f32 {
        self.raw().audioSampleRate
    }

    pub fn analog_frames(&self) -> usize {
        self.raw().analogFrames as _
    }

    pub fn analog_in_channels(&self) -> usize {
        self.raw().analogInChannels as _
    }

    pub fn analog_out_channels(&self) -> usize {
        self.raw().analogOutChannels as _
    }

    pub fn analog_sample_rate(&self) -> f32 {
        self.raw().analogSampleRate
    }

    pub fn digital_frames(&self) -> usize {
        self.raw().digitalFrames as _
    }

    pub fn digital_channels(&self) -> usize {
        self.raw().digitalChannels as _
    }

    pub fn digital_sample_rate(&self) -> f32 {
        self.raw().digitalSampleRate
    }

    pub fn audio_frames_elapsed(&self) -> usize {
        self.raw().audioFramesElapsed as _
    }

    pub fn multiplexer_channels(&self) -> usize {
        self.raw().multiplexerChannels as _
    }

    pub fn multiplexer_starting_channel(&self) -> usize {
        self.raw().multiplexerStartingChannel as _
    }

    pub fn audio_expander_enabled(&self) -> u32 {
        self.raw().audioExpanderEnabled
    }

    pub fn flags(&self) -> u32 {
        self.raw().flags
    }
}

// functions for setup contexts only
impl SetupContext {}

// functions for render contexts only
impl RenderContext {
    /// Access the audio output slice
    pub fn audio_out(&mut self) -> &mut [f32] {
        let n_frames = self.audio_frames();
        let n_channels = self.audio_out_channels();
        let audio_out_ptr = self.raw().audioOut;
        unsafe { from_raw_parts_mut(audio_out_ptr, n_frames * n_channels) }
    }

    /// Access the audio input slice
    pub fn audio_in(&self) -> &[f32] {
        let n_frames = self.audio_frames();
        let n_channels = self.audio_in_channels();
        let audio_in_ptr = self.raw().audioIn;
        unsafe { from_raw_parts(audio_in_ptr, n_frames * n_channels) }
    }

    /// Access the digital input/output slice mutably
    pub fn digital_mut(&mut self) -> &mut [u32] {
        let n_frames = self.digital_frames();
        let n_channels = self.digital_channels();
        let digital_ptr = self.raw().digital;
        unsafe { from_raw_parts_mut(digital_ptr, n_frames * n_channels) }
    }

    /// Access the digital input/output slice immutably
    pub fn digital(&self) -> &[u32] {
        let n_frames = self.digital_frames();
        let n_channels = self.digital_channels();
        let digital_ptr = self.raw().digital;
        unsafe { from_raw_parts(digital_ptr, n_frames * n_channels) }
    }

    /// Access the analog output slice
    pub fn analog_out(&mut self) -> &mut [f32] {
        let n_frames = self.analog_frames();
        let n_channels = self.analog_out_channels();
        let analog_out_ptr = self.raw().analogOut;
        unsafe { from_raw_parts_mut(analog_out_ptr, n_frames * n_channels) }
    }

    /// Access the analog input slice
    pub fn analog_in(&self) -> &[f32] {
        let n_frames = self.analog_frames();
        let n_channels = self.analog_in_channels();
        let analog_in_ptr = self.raw().analogIn;
        unsafe { from_raw_parts(analog_in_ptr, n_frames * n_channels) }
    }

    pub fn multiplexer_analog_in(&self) -> Option<&[f32]> {
        let n_frames = self.analog_frames();
        let n_channels = self.multiplexer_channels();
        let analog_in_ptr = self.raw().multiplexerAnalogIn;
        if analog_in_ptr.is_null() {
            None
        } else {
            Some(unsafe { from_raw_parts(analog_in_ptr, n_frames * n_channels) })
        }
    }

    /// Returns the value of a given digital input at the given frame number
    pub fn digital_read(&self, frame: usize, channel: usize) -> bool {
        let digital = self.digital();
        (digital[frame] >> (channel + 16)) & 1 != 0
    }

    /// Sets a given digital output channel to a value for the current frame and all subsequent frames
    pub fn digital_write(&mut self, frame: usize, channel: usize, value: bool) {
        let digital = self.digital_mut();
        for out in &mut digital[frame..] {
            if value {
                *out |= 1 << (channel + 16)
            } else {
                *out &= !(1 << (channel + 16));
            }
        }
    }

    /// Sets a given digital output channel to a value for the current frame only
    pub fn digital_write_once(&mut self, frame: usize, channel: usize, value: bool) {
        let digital = self.digital_mut();
        if value {
            digital[frame] |= 1 << (channel + 16);
        } else {
            digital[frame] &= !(1 << (channel + 16));
        }
    }

    /// Sets the direction of a digital pin for the current frame and all subsequent frames
    pub fn pin_mode(&mut self, frame: usize, channel: usize, mode: DigitalDirection) {
        let digital = self.digital_mut();
        for out in &mut digital[frame..] {
            match mode {
                DigitalDirection::Input => {
                    *out |= 1 << channel;
                }
                DigitalDirection::Output => {
                    *out &= !(1 << channel);
                }
            }
        }
    }

    /// Sets the direction of a digital pin for the current frame only
    pub fn pin_mode_once(&mut self, frame: usize, channel: usize, mode: DigitalDirection) {
        let digital = self.digital_mut();
        match mode {
            DigitalDirection::Input => {
                digital[frame] |= 1 << channel;
            }
            DigitalDirection::Output => {
                digital[frame] &= !(1 << channel);
            }
        }
    }
}
