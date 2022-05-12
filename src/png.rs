use crate::{chunk::Chunk, Error, Result};
use std::str::FromStr;

struct Png {
    chunks: Vec<Chunk>,
}

impl Png {
    const STANDARD_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

    fn from_chunks(chunks: Vec<Chunk>) -> Self {
        Png { chunks }
    }

    fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }
}

impl TryFrom<&[u8]> for Png {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let mut chunks: Vec<Chunk> = vec![];
        let header = &value[..8];
        let mut cursor = 8usize;

        while cursor < value.len() {
            let chunk = Chunk::try_from(&value[cursor..]).unwrap();

            cursor += 4 + 4 + chunk.length() as usize + 4;
            chunks.push(chunk);
        }

        Ok(Png { chunks })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chunk::Chunk, chunk_type::ChunkType, Result};

    #[test]
    fn test_from_chunks() {
        let chunks = testing_chunks();
        let png = Png::from_chunks(chunks);

        assert_eq!(png.chunks().len(), 3);
    }

    #[test]
    fn test_valid_from_bytes() {
        let chunk_bytes: Vec<u8> = testing_chunks()
            .into_iter()
            .flat_map(|chunk| chunk.as_bytes())
            .collect();
        let bytes: Vec<u8> = Png::STANDARD_HEADER
            .iter()
            .chain(chunk_bytes.iter())
            .copied()
            .collect();
        let png = Png::try_from(bytes.as_ref());

        assert!(png.is_ok());
    }

    #[test]
    fn test_invalid_header() {
        let chunk_bytes: Vec<u8> = testing_chunks()
            .into_iter()
            .flat_map(|chunk| chunk.as_bytes())
            .collect();
        let bytes: Vec<u8> = [13, 80, 78, 71, 13, 10, 26, 10]
            .iter()
            .chain(chunk_bytes.iter())
            .copied()
            .collect();
        let png = Png::try_from(bytes.as_ref());

        assert!(png.is_err());
    }

    fn testing_chunks() -> Vec<Chunk> {
        let mut chunks = Vec::new();

        chunks.push(chunk_from_strings("FrSt", "I am the first chunk").unwrap());
        chunks.push(chunk_from_strings("miDl", "I am another chunk").unwrap());
        chunks.push(chunk_from_strings("LASt", "I am the last chunk").unwrap());
        chunks
    }

    fn chunk_from_strings(chunk_type: &str, data: &str) -> Result<Chunk> {
        let chunk_type = ChunkType::from_str(chunk_type)?;
        let data: Vec<u8> = data.bytes().collect();

        Ok(Chunk::new(chunk_type, data))
    }
}
