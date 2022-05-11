use std::{error, result};

mod chunk;
mod chunk_type;
mod png;

pub type Error = Box<dyn error::Error>;
pub type Result<T> = result::Result<T, Error>;

fn main() -> Result<()> {
    todo!()
}
