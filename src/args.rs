use crate::{
    chunk::Chunk,
    chunk_type::ChunkType,
    png::{ChunkNotFoundError, Png},
    Result,
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

impl EncodeArgs {
    pub fn encode(&self) -> Result<()> {
        let mut file = Self::prepare_file(&self.file_path);
        let chunk = Self::prepare_chunk(&self.chunk_type, &self.message)?;
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

    fn prepare_chunk(chunk_type: &str, message: &str) -> Result<Chunk> {
        Ok(Chunk::new(
            ChunkType::from_str(chunk_type)?,
            message.as_bytes().to_vec(),
        ))
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

        removed_chunk
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
    fn test_encode_new_file() {
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
    fn test_encode_new_file_with_separate_output() {
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
