use std::io::Error as ioError;
use std::fmt;
use std::str;
use std::error;

#[derive(Debug)]
pub enum NsqError {
    IOError(ioError),
}

impl fmt::Display for NsqError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NsqError::IOError(ref err) => write!(f, "IO error: {}", err),
        }
    }
}

impl error::Error for NsqError {
    fn description(&self) -> &str {
        match *self {
            NsqError::IOError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            NsqError::IOError(ref err) => Some(err),          
        }
    }
}

impl From<ioError> for NsqError {
    fn from(err: ioError) -> NsqError {
        NsqError::IOError(err)
    }
}