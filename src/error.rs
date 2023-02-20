#[derive(Debug)]
pub enum SpindleError {
    Io,
    NoAccess,
    NoSpindle { id: u16 },
    NoSlot { id: u16, number: u8 },
    Busy {id: u16 },
    Timeout,
    NoMem,
    UnsupportedOperation,
    Unknown,
}

impl std::error::Error for SpindleError {}

impl std::fmt::Display for SpindleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SpindleError::Io  => write!(f, "I/O Operation Failure"),
            SpindleError::NoAccess => write!(f, "Insufficient Access"),
            SpindleError::Busy { id } => write!(f, "Spindle {} Busy", id),
            SpindleError::NoSpindle { id } => write!(f, "Spindle {} Not Found", id),
            SpindleError::NoSlot { id, number } => write!(f, "Spindle {} Slot {} Not Found", id, number),
            SpindleError::NoMem => write!(f, "Insufficient Memory"),
            SpindleError::Timeout => write!(f, "Operation timed out"),
            SpindleError::UnsupportedOperation => write!(f, "Unsupported Operation"),
            SpindleError::Unknown => write!(f, "Unknown Error"),
        }
    }
}

impl From<rusb::Error> for SpindleError {
    fn from(error: rusb::Error) -> Self {
        match error {
            rusb::Error::Io => SpindleError::Io,
            rusb::Error::NotSupported => SpindleError::UnsupportedOperation,
            rusb::Error::Access => SpindleError::NoAccess,
            _ => SpindleError::Unknown,
        }
    }
}
