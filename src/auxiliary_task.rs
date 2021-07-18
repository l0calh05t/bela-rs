/// Handle to a created auxiliary Bela task
pub struct AuxiliaryTask(bela_sys::AuxiliaryTask);

unsafe impl Send for AuxiliaryTask {}
