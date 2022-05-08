use std::{error, fmt::Display, str::FromStr};

use crate::Error;

#[derive(Debug, PartialEq)]
struct ChunkType {
    bytes: [u8; 4],
}

#[derive(Debug)]
struct InvalidChunkError;

impl ChunkType {
    fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    fn is_critical(&self) -> bool {
        /*
            from http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html#Chunk-naming-conventions

            the chunk is critical if the bit in position 5 (value 32) of the first byte is 0
        */
        Self::test_fifth_bit_to_0(self.bytes[0])
    }

    fn is_public(&self) -> bool {
        /*
            from http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html#Chunk-naming-conventions

            the chunk is public if the bit in position 5 (value 32) of the second byte is 0
        */
        Self::test_fifth_bit_to_0(self.bytes[1])
    }

    fn is_reserved_bit_valid(&self) -> bool {
        /*
            from http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html#Chunk-naming-conventions

            the chunk has a valid reserved bit if the bit in position 5 (value 32) of the third byte is 0
        */
        Self::test_fifth_bit_to_0(self.bytes[2])
    }

    fn is_safe_to_copy(&self) -> bool {
        /*
            from http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html#Chunk-naming-conventions

            the chunk is safe to copy if the bit in position 5 (value 32) of the fourth byte is 1
        */
        !Self::test_fifth_bit_to_0(self.bytes[3])
    }

    fn is_valid(&self) -> bool {
        /*
            from http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html#Chunk-layout

            the chunk is valid if all of its bytes are ASCII uppercase or lowercase letters, and also if the reserved bit is valid
        */
        self.bytes
            .iter()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_lowercase())
            && self.is_reserved_bit_valid()
    }

    fn test_fifth_bit_to_0(byte: u8) -> bool {
        byte & 0b00100000 == 0
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        Ok(Self { bytes: value })
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0u8; 4];

        for (i, b) in s.bytes().enumerate() {
            if b.is_ascii_uppercase() || b.is_ascii_lowercase() {
                bytes[i] = b;
            } else {
                /*
                    the Box is necessary because the Error alias in main.rs was defined to accept trait objects
                */
                return Err(Box::new(InvalidChunkError));
            }
        }

        Ok(Self { bytes })
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

/*
    our custom error type must implement std::error::Error (and therefore Display) to be returned inside an Err() variant
*/
impl error::Error for InvalidChunkError {}

impl Display for InvalidChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A valid chunk contains only ASCII uppercase or lowercase letters"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();

        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();

        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();

        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();

        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();

        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();

        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();

        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();

        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();

        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();

        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();

        assert_eq!(&chunk.to_string(), "RuSt");
    }
}
