use anyhow::Result;
use args::{CommandType, PngMeArgs};
use clap::Parser;

mod args;
mod chunk;
mod chunk_type;
mod png;

fn main() -> Result<()> {
    match PngMeArgs::parse().command_type {
        CommandType::Encode(encode_args) => match encode_args.encode() {
            Ok(_) => println!("Encoding successful"),
            Err(e) => eprintln!("{e}"),
        },
        CommandType::Decode(decode_args) => match decode_args.decode() {
            Ok(s) => println!("Decoded: {s}"),
            Err(e) => eprintln!("{e}"),
        },
        CommandType::Remove(remove_args) => match remove_args.remove() {
            Ok(c) => println!("Removed: {c}"),
            Err(e) => eprintln!("{e}"),
        },
        CommandType::Print(print_args) => match print_args.print() {
            Ok(p) => println!("PNG: {p}"),
            Err(e) => eprintln!("{e}"),
        },
    }

    Ok(())
}
