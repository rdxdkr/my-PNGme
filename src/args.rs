use crate::{
    chunk::Chunk,
    chunk_type::ChunkType,
    png::{ChunkNotFoundError, Png},
    Error, Result,
};
use clap::{Args, Parser, Subcommand};
use std::{
    fs::{self, File},
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

    /// Remove a PNG chunk from a file
    Remove(RemoveArgs),

    /// Print the chunks of a PNG file
    Print(PrintArgs),
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

#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// The path of the PNG file
    pub file_path: String,

    /// The type of PNG chunk to remove
    pub chunk_type: String,
}

#[derive(Debug, Args)]
pub struct PrintArgs {
    /// The path of the PNG file
    pub file_path: String,
}

enum FileState {
    Png,
    Empty,
    Other(Error),
}

impl EncodeArgs {
    pub fn encode(&self) -> Result<()> {
        let mut input_file = File::options()
            .read(true)
            .append(true)
            .create(true)
            .open(&self.file_path)?;
        let chunk = Chunk::new(
            ChunkType::from_str(&self.chunk_type)?,
            self.message.as_bytes().to_vec(),
        );
        let mut input_buffer = Vec::<u8>::new();

        input_file.read_to_end(&mut input_buffer)?;

        if let Some(output_path) = &self.output_file {
            // fill buffer according to both input and output
            let mut output_file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .open(output_path)?;
            let mut output_buffer = Vec::<u8>::new();

            output_file.read_to_end(&mut output_buffer)?;
            output_file
                .write_all(&Self::validate_input_with_output(
                    &input_buffer,
                    &output_buffer,
                    chunk,
                )?)
                .map_err(|e| e.into())
        } else {
            // fill buffer only according to input
            input_file
                .write_all(&Self::validate_input(&input_buffer, chunk)?)
                .map_err(|e| e.into())
        }
    }

    fn validate_png(input_contents: &Vec<u8>) -> FileState {
        if input_contents.is_empty() {
            FileState::Empty
        } else {
            match Png::try_from(&input_contents[..]) {
                Ok(_) => FileState::Png,
                Err(e) => FileState::Other(Box::new(e)),
            }
        }
    }

    fn validate_input_with_output(
        input_buffer: &Vec<u8>,
        output_buffer: &Vec<u8>,
        chunk: Chunk,
    ) -> Result<Vec<u8>> {
        match (
            Self::validate_png(input_buffer),
            Self::validate_png(output_buffer),
        ) {
            (FileState::Png, FileState::Empty) => {
                // valid input, empty output
                let mut png = Png::try_from(&input_buffer[..])?;

                png.append_chunk(chunk);
                Ok(png.as_bytes().to_vec())
            }
            (FileState::Empty, FileState::Empty) => {
                // empty input, empty output
                Ok(Png::from_chunks(vec![chunk]).as_bytes().to_vec())
            }
            (FileState::Png, FileState::Png) => todo!(), // valid input, valid output
            (FileState::Empty, FileState::Png) => todo!(), // empty input, valid output
            (FileState::Other(e), _) | (_, FileState::Other(e)) => Err(e), // invalid input or output
        }
    }

    fn validate_input(input_buffer: &Vec<u8>, chunk: Chunk) -> Result<Vec<u8>> {
        match Self::validate_png(input_buffer) {
            FileState::Png => Ok(chunk.as_bytes().to_vec()), // valid input
            FileState::Empty => Ok(Png::from_chunks(vec![chunk]).as_bytes().to_vec()), // empty input
            FileState::Other(e) => Err(e), // invalid input
        }
    }
}

impl DecodeArgs {
    pub fn decode(&self) -> Result<String> {
        let buffer = fs::read(&self.file_path)?;
        let png = Png::try_from(&buffer[..])?;

        match png.chunk_by_type(&self.chunk_type) {
            Some(data) => data.data_as_string(),
            None => Err(Box::new(ChunkNotFoundError)),
        }
    }
}

impl RemoveArgs {
    pub fn remove(&self) -> Result<Chunk> {
        let buffer = fs::read(&self.file_path)?;
        let mut png = Png::try_from(&buffer[..])?;
        let removed_chunk = png.remove_chunk(&self.chunk_type);

        if png.chunks().is_empty() {
            fs::remove_file(&self.file_path).unwrap();
        } else if removed_chunk.is_ok() {
            fs::write(&self.file_path, &png.as_bytes()[..]).unwrap();
        }

        removed_chunk.map_err(|e| Box::new(e) as Box<dyn crate::error::Error>)
    }
}

impl PrintArgs {
    pub fn print(&self) -> Result<String> {
        let buffer = fs::read(&self.file_path)?;

        Ok(Png::try_from(&buffer[..])?.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chunk::Chunk, chunk_type::ChunkType, png::Png};
    use std::{
        fs::{self, File},
        str::FromStr,
    };

    /*
        since these tests involve file manipulation, either each test has to work with a different
        file or the tests must not be run concurrently to avoid unexpected behaviour

        in this case, the tests are run with "cargo test -- --test-threads=1"
    */

    const FILE_NAME: &str = "test.png";
    const OUTPUT_NAME: &str = "output.png";
    const INVALID_FILE_NAME: &str = "invalid.png";

    #[test]
    fn test_encode_empty_file() {
        File::create(FILE_NAME).unwrap();

        EncodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
            message: String::from("I am the first chunk"),
            output_file: None,
        }
        .encode()
        .unwrap();

        let png_from_file = Png::try_from(&fs::read(FILE_NAME).unwrap()[..]).unwrap();

        assert_eq!(png_from_file.as_bytes(), testing_png_simple().as_bytes());
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_encode_creates_new_file_if_not_exists() {
        EncodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
            message: String::from("I am the first chunk"),
            output_file: None,
        }
        .encode()
        .unwrap();

        let png_from_file = Png::try_from(&fs::read(FILE_NAME).unwrap()[..]).unwrap();

        assert_eq!(png_from_file.as_bytes(), testing_png_simple().as_bytes());
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_encode_existing_file() {
        prepare_file(FILE_NAME);

        let new_chunk = testing_chunk().unwrap();

        EncodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: new_chunk.chunk_type().to_string(),
            message: new_chunk.data_as_string().unwrap(),
            output_file: None,
        }
        .encode()
        .unwrap();

        let png_from_file = Png::try_from(&fs::read(FILE_NAME).unwrap()[..]).unwrap();

        assert_eq!(
            png_from_file.as_bytes(),
            testing_png_full()
                .as_bytes()
                .iter()
                .chain(new_chunk.as_bytes().iter())
                .cloned()
                .collect::<Vec<u8>>()
        );
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_encode_empty_file_with_separate_output() {
        File::create(FILE_NAME).unwrap();
        EncodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
            message: String::from("I am the first chunk"),
            output_file: Some(String::from(OUTPUT_NAME)),
        }
        .encode()
        .unwrap();
        assert!(fs::read(FILE_NAME).unwrap().is_empty());

        let png_from_empty_file = Png::try_from(&fs::read(FILE_NAME).unwrap()[..]);
        let png_from_output_file = Png::try_from(&fs::read(OUTPUT_NAME).unwrap()[..]).unwrap();

        assert!(png_from_empty_file.is_err());
        assert_eq!(
            png_from_output_file.as_bytes(),
            testing_png_simple().as_bytes()
        );
        fs::remove_file(FILE_NAME).unwrap();
        fs::remove_file(OUTPUT_NAME).unwrap();
    }

    #[test]
    fn test_encode_existing_file_with_separate_output() {
        prepare_file(FILE_NAME);

        let new_chunk = testing_chunk().unwrap();

        EncodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: new_chunk.chunk_type().to_string(),
            message: new_chunk.data_as_string().unwrap(),
            output_file: Some(String::from(OUTPUT_NAME)),
        }
        .encode()
        .unwrap();

        let png_from_input_file = Png::try_from(&fs::read(FILE_NAME).unwrap()[..]).unwrap();
        let png_from_output_file = Png::try_from(&fs::read(OUTPUT_NAME).unwrap()[..]).unwrap();

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
        fs::remove_file(FILE_NAME).unwrap();
        fs::remove_file(OUTPUT_NAME).unwrap();
    }

    #[test]
    fn test_encode_chunk_type_too_long() {
        let result = EncodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("abcdefg"),
            message: String::from("My chunk type is invalid"),
            output_file: None,
        }
        .encode();

        assert!(result.is_err());
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_decode_existing_file() {
        prepare_file(FILE_NAME);

        let decode_args = DecodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
        };

        assert_eq!(decode_args.decode().unwrap(), "I am the first chunk");
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_decode_does_not_modify_input_file() {
        prepare_file(FILE_NAME);

        DecodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
        }
        .decode()
        .unwrap();

        let png_from_input_file = Png::try_from(&fs::read(FILE_NAME).unwrap()[..]).unwrap();

        assert_eq!(
            png_from_input_file.as_bytes(),
            testing_png_full().as_bytes()
        );
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_decode_non_existing_file() {
        let decode_args = DecodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
        };

        assert!(decode_args.decode().is_err());
    }

    #[test]
    fn test_decode_invalid_file() {
        File::create(INVALID_FILE_NAME).unwrap();

        let decode_args = DecodeArgs {
            file_path: String::from(INVALID_FILE_NAME),
            chunk_type: String::from("FrSt"),
        };

        assert!(decode_args.decode().is_err());
        fs::remove_file(INVALID_FILE_NAME).unwrap();
    }

    #[test]
    fn test_decode_valid_file_without_required_chunk() {
        prepare_file(FILE_NAME);

        let decode_args = DecodeArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("TeSt"),
        };

        assert!(decode_args.decode().is_err());
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_remove_existing_file() {
        prepare_file(FILE_NAME);

        let remove_args = RemoveArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
        };
        let removed_chunk = remove_args.remove().unwrap();
        let testing_chunk = chunk_from_strings("FrSt", "I am the first chunk").unwrap();

        assert_eq!(removed_chunk.as_bytes(), testing_chunk.as_bytes());
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_remove_does_modify_input_file() {
        prepare_file(FILE_NAME);

        let remove_args = RemoveArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
        };
        let mut png = testing_png_full();

        remove_args.remove().unwrap();
        png.remove_chunk("FrSt").unwrap();

        let png_from_file = Png::try_from(&fs::read(FILE_NAME).unwrap()[..]).unwrap();

        assert_eq!(png.as_bytes(), png_from_file.as_bytes());
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_remove_non_existing_file() {
        let remove_args = RemoveArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
        };

        assert!(remove_args.remove().is_err());
    }

    #[test]
    fn test_remove_invalid_file() {
        File::create(INVALID_FILE_NAME).unwrap();

        let remove_args = RemoveArgs {
            file_path: String::from(INVALID_FILE_NAME),
            chunk_type: String::from("FrSt"),
        };

        assert!(remove_args.remove().is_err());
        fs::remove_file(INVALID_FILE_NAME).unwrap();
    }

    #[test]
    fn test_remove_valid_file_without_required_chunk() {
        prepare_file(FILE_NAME);

        let remove_args = RemoveArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("TeSt"),
        };
        let result = remove_args.remove();
        let png_from_file = Png::try_from(&fs::read(FILE_NAME).unwrap()[..]).unwrap();

        assert!(result.is_err());
        assert_eq!(png_from_file.as_bytes(), testing_png_full().as_bytes());
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_remove_deletes_file_after_removing_last_chunk() {
        File::create(FILE_NAME).unwrap();
        fs::write(FILE_NAME, testing_png_simple().as_bytes()).unwrap();

        let remove_args = RemoveArgs {
            file_path: String::from(FILE_NAME),
            chunk_type: String::from("FrSt"),
        };

        remove_args.remove().unwrap();
        assert!(File::open(FILE_NAME).is_err());
    }

    #[test]
    fn test_print_existing_file() {
        prepare_file(FILE_NAME);

        let print_args = PrintArgs {
            file_path: String::from(FILE_NAME),
        };

        assert_eq!(print_args.print().unwrap(), testing_png_full().to_string());
        fs::remove_file(FILE_NAME).unwrap();
    }

    #[test]
    fn test_print_non_existing_file() {
        let print_args = PrintArgs {
            file_path: String::from(FILE_NAME),
        };

        assert!(print_args.print().is_err());
    }

    #[test]
    fn test_print_invalid_file() {
        File::create(INVALID_FILE_NAME).unwrap();

        let print_args = PrintArgs {
            file_path: String::from(INVALID_FILE_NAME),
        };

        assert!(print_args.print().is_err());
        fs::remove_file(INVALID_FILE_NAME).unwrap();
    }

    fn prepare_file(file_name: &str) {
        let png = testing_png_full();

        fs::write(file_name, &png.as_bytes()).unwrap();
    }

    fn testing_chunk() -> Result<Chunk> {
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
