use std::{error, io};

pub fn box_string_error(string: &str) -> Box<dyn error::Error> {
    Box::new(io::Error::new(io::ErrorKind::Other,string))
}


