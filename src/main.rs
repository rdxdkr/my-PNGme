use args::PngMeArgs;
use clap::Parser;
use std::{error, result};

mod args;
mod chunk;
mod chunk_type;
mod png;

pub type Error = Box<dyn error::Error>;
pub type Result<T> = result::Result<T, Error>;

fn main() -> Result<()> {
    let args = PngMeArgs::parse();

    dbg!(args);
    Ok(())
}
