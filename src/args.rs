use crate::{chunk::Chunk, chunk_type::ChunkType, png::Png, Result};
use clap::{Args, Parser, Subcommand};
use std::{
    fs::File,
    io::{Read, Write},
    str::FromStr,
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct PngMeArgs {
    #[clap(subcommand)]
    pub command_type: CommandType,
}

#[derive(Debug, Subcommand)]
pub enum CommandType {
    /// Encode a message in a PNG chunk and save it in a file
    Encode(EncodeArgs),

    /// Decode a message from a PNG chunk contained in a file
    Decode(DecodeArgs),
}

#[derive(Debug, Args)]
pub struct EncodeArgs {
    /// The path of the PNG file
    pub file_path: String,

    /// The type of PNG chunk in which to encode the message
    pub chunk_type: String,

    /// The message to encode
    pub message: String,

    /// The optional path in which to save the resulting PNG file
    pub output_file: Option<String>,
}

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// The path of the PNG file
    pub file_path: String,

    /// The type of PNG chunk to decode
    pub chunk_type: String,
}

impl EncodeArgs {
    fn encode(&self) -> Result<()> {
        let mut file = Self::prepare_file(&self.file_path);
        let chunk = Self::prepare_chunk(&self.chunk_type, &self.message);
        let (png, bytes_read) = Self::prepare_png(&mut file, chunk);
        let buffer = if bytes_read == 0 || self.output_file.is_some() {
            png.as_bytes()
        } else {
            png.chunk_by_type(&self.chunk_type).unwrap().as_bytes()
        };
        let mut output_file = match &self.output_file {
            Some(file_name) => File::create(file_name).unwrap(),
            None => file,
        };

        // if a file with the given name does not contain a valid PNG structure, do I need to overwrite it all?
        match output_file.write_all(&buffer) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn prepare_file(file_path: &str) -> File {
        // if a file with the given name is found, open it, else create a new one
        if let Ok(file) = File::options().read(true).append(true).open(file_path) {
            file
        } else {
            File::create(file_path).unwrap()
        }
    }

    fn prepare_chunk(chunk_type: &str, message: &str) -> Chunk {
        Chunk::new(
            ChunkType::from_str(chunk_type).unwrap(),
            message.as_bytes().to_vec(),
        )
    }

    fn prepare_png(file: &mut File, chunk: Chunk) -> (Png, usize) {
        let mut buffer = Vec::<u8>::new();
        let mut bytes_read = 0;

        /*
            if a file with the given name already exists but it's empty,
            write a full PNG inside it, else append just the new chunk
        */
        if let Ok(bytes) = file.read_to_end(&mut buffer) {
            bytes_read = bytes;

            if let Ok(mut png) = Png::try_from(&buffer[..]) {
                png.append_chunk(chunk);
                return (png, bytes_read);
            }
        }

        (Png::from_chunks(vec![chunk]), bytes_read)
    }
}

impl DecodeArgs {
    fn decode(&self) -> Result<String> {
        let mut file = File::open(&self.file_path)?;
        let mut buffer = Vec::<u8>::new();
        
        file.read_to_end(&mut buffer).unwrap();

        let png = Png::try_from(&buffer[..])?;

        png.chunk_by_type(&self.chunk_type).unwrap().data_as_string()
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

    /*
        since these tests involve file manipulation, either each test has to work with a different
        file or the tests must not be run concurrently to avoid unexpected behaviour

        in this case, the tests are run with "cargo test -- --test-threads=1"
    */

    const FILE_NAME: &str = "test.png";
    const OUTPUT_NAME: &str = "output.png";
    const ENCODE: &str = "encode";
    const DECODE: &str = "decode";

    #[test]
    fn test_encode_new_file() {
        create_file(FILE_NAME);

        let args = parse_args(&[ENCODE, FILE_NAME, "FrSt", "I am the first chunk"]).unwrap();

        if let CommandType::Encode(encode_args) = args.command_type {
            encode_args.encode().unwrap();

            let png_from_file = Png::try_from(&read_file(FILE_NAME)[..]).unwrap();

            assert_eq!(png_from_file.as_bytes(), testing_png_simple().as_bytes());
            delete_file(FILE_NAME);
        }
    }

    #[test]
    fn test_encode_creates_new_file_if_not_exists() {
        let args = parse_args(&[ENCODE, FILE_NAME, "FrSt", "I am the first chunk"]).unwrap();

        if let CommandType::Encode(encode_args) = args.command_type {
            encode_args.encode().unwrap();

            let png_from_file = Png::try_from(&read_file(FILE_NAME)[..]).unwrap();

            assert_eq!(png_from_file.as_bytes(), testing_png_simple().as_bytes());
            delete_file(FILE_NAME);
        }
    }

    #[test]
    fn test_encode_existing_file() {
        prepare_file(FILE_NAME);

        let new_chunk = test_chunk().unwrap();
        let args = parse_args(&[
            ENCODE,
            FILE_NAME,
            &new_chunk.chunk_type().to_string(),
            &new_chunk.data_as_string().unwrap(),
        ])
        .unwrap();

        if let CommandType::Encode(encode_args) = args.command_type {
            encode_args.encode().unwrap();

            let png_from_file = Png::try_from(&read_file(FILE_NAME)[..]).unwrap();

            assert_eq!(
                png_from_file.as_bytes(),
                testing_png_full()
                    .as_bytes()
                    .iter()
                    .chain(new_chunk.as_bytes().iter())
                    .cloned()
                    .collect::<Vec<u8>>()
            );
            delete_file(FILE_NAME);
        }
    }

    #[test]
    fn test_encode_new_file_with_separate_output() {
        create_file(FILE_NAME);

        let args = parse_args(&[
            ENCODE,
            FILE_NAME,
            "FrSt",
            "I am the first chunk",
            OUTPUT_NAME,
        ])
        .unwrap();

        if let CommandType::Encode(encode_args) = args.command_type {
            encode_args.encode().unwrap();

            let empty_input_file = read_file(FILE_NAME);
            let png_from_empty_file = Png::try_from(&read_file(FILE_NAME)[..]);

            assert_eq!(empty_input_file.len(), 0);
            assert!(png_from_empty_file.is_err());

            let png_from_output_file = Png::try_from(&read_file(OUTPUT_NAME)[..]).unwrap();

            assert_eq!(
                png_from_output_file.as_bytes(),
                testing_png_simple().as_bytes()
            );
            delete_file(FILE_NAME);
            delete_file(OUTPUT_NAME);
        }
    }

    #[test]
    fn test_encode_existing_file_with_separate_output() {
        prepare_file(FILE_NAME);

        let new_chunk = test_chunk().unwrap();
        let args =
            parse_args(&[ENCODE, FILE_NAME, "TeSt", "I am a test chunk", OUTPUT_NAME]).unwrap();

        if let CommandType::Encode(encode_args) = args.command_type {
            encode_args.encode().unwrap();

            let png_from_input_file = Png::try_from(&read_file(FILE_NAME)[..]).unwrap();
            let png_from_output_file = Png::try_from(&read_file(OUTPUT_NAME)[..]).unwrap();

            assert_eq!(
                png_from_input_file.as_bytes(),
                testing_png_full().as_bytes()
            );
            assert_eq!(
                png_from_output_file.as_bytes(),
                testing_png_full()
                    .as_bytes()
                    .iter()
                    .chain(new_chunk.as_bytes().iter())
                    .cloned()
                    .collect::<Vec<u8>>()
            );
            delete_file(FILE_NAME);
            delete_file(OUTPUT_NAME);
        }
    }

   #[test]
    fn test_decode_existing_file() {
        prepare_file(FILE_NAME);

        let args = parse_args(&[DECODE, FILE_NAME, "FrSt"]).unwrap();

        if let CommandType::Decode(decode_args) = args.command_type {
            let message = decode_args.decode();

            assert!(message.is_ok());
            assert_eq!(message.unwrap(), "I am the first chunk");

            // SPOSTARE FUORI TUTTI I DELETE FILE ANCHE SOPRA
            delete_file(FILE_NAME);
        }
    }

    #[test]
    fn test_decode_does_not_modify_input_file() {
        prepare_file(FILE_NAME);

        let args = parse_args(&[DECODE, FILE_NAME, "FrSt"]).unwrap();

        if let CommandType::Decode(_) = args.command_type {
            let png_from_input_file = Png::try_from(&read_file(FILE_NAME)[..]).unwrap();

            assert_eq!(
                png_from_input_file.as_bytes(),
                testing_png_full().as_bytes()
            );

            delete_file(FILE_NAME);
        }
    }

    #[test]
    fn test_decode_non_existing_file() {
        let args = parse_args(&[DECODE, FILE_NAME, "FrSt"]).unwrap();

        if let CommandType::Decode(decode_args) = args.command_type {
            let message = decode_args.decode();

            assert!(message.is_err());
        }
    }

    #[test]
    fn test_decode_invalid_file() {
        const INVALID_FILE_NAME: &str = "invalid_png";

        let file = File::create(INVALID_FILE_NAME).unwrap();
        let args = parse_args(&[DECODE, INVALID_FILE_NAME, "FrSt"]).unwrap();

        if let CommandType::Decode(decode_args) = args.command_type {
            let message = decode_args.decode();

            assert!(message.is_err());
            delete_file(INVALID_FILE_NAME);
        }
    }

    fn create_file(file_name: &str) {
        File::create(file_name).unwrap();
    }

    fn prepare_file(file_name: &str) {
        create_file(file_name);

        let png = testing_png_full();
        let mut file = File::options().write(true).open(file_name).unwrap();

        file.write_all(&png.as_bytes()).unwrap();
    }

    fn read_file(file_name: &str) -> Vec<u8> {
        let mut buffer = Vec::<u8>::new();
        let mut file = File::open(file_name).unwrap();

        file.read_to_end(&mut buffer).unwrap();
        buffer
    }

    fn delete_file(file_name: &str) {
        fs::remove_file(file_name).unwrap();
    }

    fn parse_args(args: &[&str]) -> clap::Result<PngMeArgs> {
        PngMeArgs::try_parse_from(std::iter::once("pngme").chain(args.iter().cloned()))
    }

    fn test_chunk() -> Result<Chunk> {
        chunk_from_strings("TeSt", "I am a test chunk")
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

    fn testing_png_full() -> Png {
        let chunks = vec![
            chunk_from_strings("FrSt", "I am the first chunk").unwrap(),
            chunk_from_strings("miDl", "I am another chunk").unwrap(),
            chunk_from_strings("LASt", "I am the last chunk").unwrap(),
        ];

        Png::from_chunks(chunks)
    }
}
