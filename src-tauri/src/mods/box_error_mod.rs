use std::{error, io};

pub fn box_string_error<S: AsRef<str>>(string: S) -> Box<dyn error::Error> {
    Box::new(io::Error::new(io::ErrorKind::Other,string.as_ref()))
}
