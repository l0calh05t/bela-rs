#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum BelaHw {
    NoHw = bela_sys::BelaHw_BelaHw_NoHw as isize,
    Bela = bela_sys::BelaHw_BelaHw_Bela as isize,
    BelaMini = bela_sys::BelaHw_BelaHw_BelaMini as isize,
    Salt = bela_sys::BelaHw_BelaHw_Salt as isize,
    CtagFace = bela_sys::BelaHw_BelaHw_CtagFace as isize,
    CtagBeast = bela_sys::BelaHw_BelaHw_CtagBeast as isize,
    CtagFaceBela = bela_sys::BelaHw_BelaHw_CtagFaceBela as isize,
    CtagBeastBela = bela_sys::BelaHw_BelaHw_CtagBeastBela as isize,
}
