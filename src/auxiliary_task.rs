use std::ffi::c_void;
use std::panic::catch_unwind;

use crate::{Error, RenderContext, SetupContext};

/// Handle to a created auxiliary Bela task
pub struct AuxiliaryTask(bela_sys::AuxiliaryTask);

unsafe impl Send for AuxiliaryTask {}

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

impl RenderContext {
    /// Schedule a created auxiliary task
    pub fn schedule_auxiliary_task(&mut self, task: &AuxiliaryTask) -> Result<(), Error> {
        let res = unsafe { bela_sys::Bela_scheduleAuxiliaryTask(task.0) };

        match res {
            0 => Ok(()),
            _ => Err(Error::ScheduleTask),
        }
    }
}
