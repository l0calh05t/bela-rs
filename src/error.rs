#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Error {
    Init,
    Start,
    CreateTask,
    ScheduleTask,
    #[cfg(feature = "midi")]
    Midi,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Error: {:?}.", self)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Init => "Bela_initAudio error",
            Error::Start => "Bela_startAudio error",
            Error::CreateTask => "Bela_createAuxiliaryTask error",
            Error::ScheduleTask => "Bela_scheduleAuxiliaryTask error",
            #[cfg(feature = "midi")]
            Error::Midi => "Midi_new error",
        }
    }
}
