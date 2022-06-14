use std::{fmt::Display, str, str::FromStr};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct ChunkType {
    bytes: [u8; 4],
}

#[derive(Debug, Error)]
#[error("A valid chunk contains only ASCII uppercase or lowercase letters")]
pub struct InvalidChunkError;

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
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
    type Error = InvalidChunkError;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        Ok(Self { bytes: value })
    }
}

impl FromStr for ChunkType {
    type Err = InvalidChunkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 4
            || s.chars()
                .any(|c| !c.is_ascii_lowercase() && !c.is_ascii_uppercase())
        {
            return Err(InvalidChunkError);
        }

        let mut bytes = [0u8; 4];

        for (i, b) in s.bytes().enumerate() {
            bytes[i] = b;
        }

        Ok(Self { bytes })
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        /*
            ASCII values can be safely cast to UTF-8 chars
        */
        //write!(f, "{}{}{}{}", self.bytes[0] as char, self.bytes[1] as char, self.bytes[2] as char, self.bytes[3] as char)

        write!(f, "{}", str::from_utf8(&self.bytes).unwrap())
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

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }

    #[test]
    pub fn test_chunk_type_too_short_is_rejected() {
        assert!(ChunkType::from_str("").is_err());
        assert!(ChunkType::from_str("R").is_err());
        assert!(ChunkType::from_str("Ru").is_err());
        assert!(ChunkType::from_str("RuS").is_err());
    }

    #[test]
    pub fn test_chunk_type_too_long_is_rejected() {
        let result = ChunkType::from_str("abcdefg");

        assert!(result.is_err());
    }
}
