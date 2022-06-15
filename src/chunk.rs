use crate::chunk_type::{ChunkType, ChunkTypeError};
use anyhow::Result;
use crc::{Crc, CRC_32_ISO_HDLC};
use std::{
    fmt::Display,
    io::{self, BufReader, Read},
    str,
};
use thiserror::Error;

pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    crc: u32,
}

#[derive(Debug, Error)]
pub enum ChunkError {
    #[error("A valid checksum must match the one that is calculated again upon creating a Chunk")]
    InvalidChecksumError,
    #[error("IO Error converting from bytes: {0}")]
    MalformedChunk(#[from] io::Error),
    #[error("Invalid ChunkType: {0}")]
    InvalidChunkType(#[from] ChunkTypeError),
}

impl Chunk {
    const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc = Self::calculate_crc(&chunk_type, &data);

        Self {
            length: data.len() as u32,
            chunk_type,
            chunk_data: data,
            crc,
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    fn data(&self) -> &[u8] {
        &self.chunk_data
    }

    pub fn data_as_string(&self) -> Result<String> {
        Ok(str::from_utf8(&self.chunk_data).unwrap().to_string())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        // this code is the same as the one used in testing_chunk() in the unit tests
        self.length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.chunk_data.iter())
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect::<Vec<u8>>()
    }

    fn calculate_crc(chunk_type: &ChunkType, data: &[u8]) -> u32 {
        /*
            from http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html#Chunk-layout
            and https://reveng.sourceforge.io/crc-catalogue/all.htm

            the crc is calculated on the bytes of the chunk type and data, and it needs to be 4 bytes long
        */
        Self::CRC.checksum(&[&chunk_type.bytes()[..], data].concat())
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Chunk {{",)?;
        writeln!(f, "  Length: {}", self.length())?;
        writeln!(f, "  Type: {}", self.chunk_type())?;
        writeln!(f, "  Data: {} bytes", self.data().len())?;
        writeln!(f, "  Crc: {}", self.crc())?;
        writeln!(f, "}}",)?;
        Ok(())
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        /*
            a slice of u8 (byte) interpreted as a png chunk is structured as follows:
            - first 4 bytes: length (n)
            - next 4 bytes: chunk type
            - next n bytes: chunk data
            - last 4 bytes: crc
        */

        let mut input_stream = BufReader::new(value);
        let mut buffer_4_bytes = [0u8; 4];

        input_stream.read_exact(&mut buffer_4_bytes)?;

        let length = u32::from_be_bytes(buffer_4_bytes);

        input_stream.read_exact(&mut buffer_4_bytes).unwrap();

        let chunk_type = ChunkType::try_from(buffer_4_bytes)?;
        let mut chunk_data = vec![0u8; length as usize];

        input_stream.read_exact(&mut chunk_data).unwrap();
        input_stream.read_exact(&mut buffer_4_bytes).unwrap();

        let input_crc = u32::from_be_bytes(buffer_4_bytes);

        if input_crc != Self::calculate_crc(&chunk_type, &chunk_data) {
            return Err(ChunkError::InvalidChecksumError);
        }

        Ok(Chunk {
            length,
            chunk_type,
            chunk_data,
            crc: input_crc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();

        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();

        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();

        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;
        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_from_bytes_invalid_length() {
        assert!(Chunk::try_from(b"0".as_ref()).is_err());
        assert!(Chunk::try_from(b"00".as_ref()).is_err());
        assert!(Chunk::try_from(b"000".as_ref()).is_err());
    }

    #[test]
    fn test_chunk_from_bytes_invalid_chunk_type() {
        let data_length: u32 = 42;
        let chunk_type = b"0000";
        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .copied()
            .collect();

        assert!(Chunk::try_from(chunk_data.as_ref()).is_err());
    }

    #[test]
    fn test_chunk_from_bytes_invalid_crc() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;
        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;
        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();
        let _chunk_string = format!("{}", chunk);
    }

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;
        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }
}
