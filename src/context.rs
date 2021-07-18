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
    pub(crate) fn raw_mut(&mut self) -> &mut bela_sys::BelaContext {
        unsafe { &mut *self.0 }
    }

    /// Reference to wrapped C struct
    pub(crate) fn raw(&self) -> &bela_sys::BelaContext {
        unsafe { &*self.0 }
    }
}

// functions for setup contexts only
impl SetupContext {}

// functions for render contexts only
impl RenderContext {
    /// Access the audio output slice
    pub fn audio_out(&mut self) -> &mut [f32] {
        let context = self.raw_mut();
        let n_frames = context.audioFrames as usize;
        let n_channels = context.audioOutChannels as usize;
        let audio_out_ptr = context.audioOut;
        unsafe { from_raw_parts_mut(audio_out_ptr, n_frames * n_channels) }
    }

    /// Access the audio input slice
    pub fn audio_in(&self) -> &[f32] {
        let context = self.raw();
        let n_frames = context.audioFrames as usize;
        let n_channels = context.audioOutChannels as usize;
        let audio_in_ptr = context.audioIn;
        unsafe { from_raw_parts(audio_in_ptr, n_frames * n_channels) }
    }
}
