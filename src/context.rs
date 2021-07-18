use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::catch_unwind;
use std::slice::{from_raw_parts, from_raw_parts_mut};

use crate::Error;

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

/// Handle to a created auxiliary Bela task
pub struct AuxiliaryTask(bela_sys::AuxiliaryTask);

unsafe impl Send for AuxiliaryTask {}

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
impl SetupContext {
    /// Create an auxiliary task that runs on a lower-priority thread
    ///
    /// # Safety
    /// `name` must be globally unique across all Xenomai processes, which cannot be verified
    /// at compile time
    pub unsafe fn create_auxiliary_task<Auxiliary>(
        &mut self, // unused reference to SetupContext, as this should only be called in Setup
        task: Box<Auxiliary>,
        priority: i32,
        name: &std::ffi::CStr,
    ) -> Result<AuxiliaryTask, Error>
    where
        Auxiliary: FnMut() + Send + 'static,
    {
        // TODO: Bela API does not currently offer an API to stop and unregister a task,
        // so we can only leak the task. Otherwise, we could `Box::into_raw` here, store the
        // raw pointer in `AuxiliaryTask` and drop it after unregistering & joining the thread
        // using `Box::from_raw`.
        let task_ptr = Box::leak(task) as *mut _ as *mut _;

        extern "C" fn auxiliary_task_trampoline<Auxiliary>(aux_ptr: *mut c_void)
        where
            Auxiliary: FnMut() + Send + 'static,
        {
            let _ = catch_unwind(|| {
                let task_ptr = unsafe { &mut *(aux_ptr as *mut Auxiliary) };
                task_ptr();
            });
        }

        // let's be explicit about which part is actually unsafe here
        #[allow(unused_unsafe)]
        let aux_task = unsafe {
            bela_sys::Bela_createAuxiliaryTask(
                Some(auxiliary_task_trampoline::<Auxiliary>),
                priority,
                name.as_ptr(),
                task_ptr,
            )
        };

        if aux_task.is_null() {
            Err(Error::CreateTask)
        } else {
            Ok(AuxiliaryTask(aux_task))
        }
    }
}

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

    /// Schedule a created auxiliary task
    pub fn schedule_auxiliary_task(&mut self, task: &AuxiliaryTask) -> Result<(), Error> {
        let res = unsafe { bela_sys::Bela_scheduleAuxiliaryTask(task.0) };

        match res {
            0 => Ok(()),
            _ => Err(Error::ScheduleTask),
        }
    }
}
