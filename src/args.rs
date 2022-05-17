use crate::{chunk::Chunk, chunk_type::ChunkType, png::Png, Result};
use clap::{Args, Parser, Subcommand};
use std::{fs::File, io::Write, str::FromStr};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct PngMeArgs {
    #[clap(subcommand)]
    pub command_type: CommandType,
}

#[derive(Debug, Subcommand)]
pub enum CommandType {
    /// Encode a message in a PNG chunk
    Encode(EncodeArgs),
}

#[derive(Debug, Args)]
pub struct EncodeArgs {
    /// The path of the PNG file
    pub file_path: String,

    /// The type of PNG chunk in which to encode the message
    pub chunk_type: String,

    /// The message to encode
    pub message: String,
}

impl EncodeArgs {
    fn encode(&self) -> Result<()> {
        let mut file = File::create(&self.file_path).unwrap();
        let chunk = Chunk::new(
            ChunkType::from_str(&self.chunk_type).unwrap(),
            self.message.as_bytes().to_vec(),
        );
        let png = Png::from_chunks(vec![chunk]);

        if let Err(e) = file.write_all(&png.as_bytes()) {
            Err(Box::new(e))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chunk::Chunk, chunk_type::ChunkType, png::Png};
    use std::{
        fs::{self, File},
        io::Read,
        str::FromStr,
    };

    #[test]
    fn test_encode_new_file() {
        let filename = "a.png";

        create_file(filename);

        let args = parse_args(&["encode", filename, "FrSt", "I am the first chunk"]).unwrap();

        if let CommandType::Encode(encode_args) = args.command_type {
            encode_args.encode().unwrap();

            let png_from_file = Png::try_from(&read_file(filename)[..]).unwrap();
            let png_test = testing_png_simple();

            assert_eq!(png_from_file.as_bytes(), png_test.as_bytes());
            delete_file(filename);
        }
    }

    fn create_file(filename: &str) {
        File::create(filename).unwrap();
    }

    fn read_file(filename: &str) -> Vec<u8> {
        let mut buffer = Vec::<u8>::new();
        let mut file = File::open(filename).unwrap();

        file.read_to_end(&mut buffer).unwrap();
        buffer
    }

    fn delete_file(filename: &str) {
        fs::remove_file(filename).unwrap();
    }

    fn parse_args(args: &[&str]) -> clap::Result<PngMeArgs> {
        PngMeArgs::try_parse_from(std::iter::once("pngme").chain(args.iter().cloned()))
    }

    fn chunk_from_strings(chunk_type: &str, data: &str) -> Result<Chunk> {
        let chunk_type = ChunkType::from_str(chunk_type)?;
        let data: Vec<u8> = data.bytes().collect();

        Ok(Chunk::new(chunk_type, data))
    }

    fn testing_png_simple() -> Png {
        let chunks = vec![chunk_from_strings("FrSt", "I am the first chunk").unwrap()];

        Png::from_chunks(chunks)
    }
}
