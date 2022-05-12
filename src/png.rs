use crate::{chunk::Chunk, Error, Result};
use std::{error, fmt::Display, str::FromStr};

struct Png {
    chunks: Vec<Chunk>,
}

#[derive(Debug)]
struct InvalidHeaderError;

impl Png {
    const STANDARD_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

    fn from_chunks(chunks: Vec<Chunk>) -> Self {
        Png { chunks }
    }

    fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    fn chunk_by_type(&self, chunk_type: &str) -> Option<&Chunk> {
        todo!()
    }
}

impl TryFrom<&[u8]> for Png {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let mut chunks: Vec<Chunk> = vec![];
        let header = &value[..8];

        if header != Self::STANDARD_HEADER {
            return Err(Box::new(InvalidHeaderError));
        }

        let mut cursor = 8usize;

        while cursor < value.len() {
            let chunk = Chunk::try_from(&value[cursor..]).unwrap();

            cursor += 4 + 4 + chunk.length() as usize + 4;
            chunks.push(chunk);
        }

        Ok(Png { chunks })
    }
}

impl error::Error for InvalidHeaderError {}

impl Display for InvalidHeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A valid PNG header must match the following sequence of bytes: [137, 80, 78, 71, 13, 10, 26, 10]"
        )
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

    #[test]
    fn test_invalid_chunk() {
        let mut chunk_bytes: Vec<u8> = testing_chunks()
            .into_iter()
            .flat_map(|chunk| chunk.as_bytes())
            .collect();

        #[rustfmt::skip]
        let mut bad_chunk = vec![
            0, 0, 0, 5,         // length
            32, 117, 83, 116,   // Chunk Type (bad)
            65, 64, 65, 66, 67, // Data
            1, 2, 3, 4, 5       // CRC (bad)
        ];

        chunk_bytes.append(&mut bad_chunk);

        let png = Png::try_from(chunk_bytes.as_ref());

        assert!(png.is_err());
    }

    #[test]
    fn test_list_chunks() {
        let png = testing_png();
        let chunks = png.chunks();

        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_chunk_by_type() {
        let png = testing_png();
        let chunk = png.chunk_by_type("FrSt").unwrap();

        assert_eq!(&chunk.chunk_type().to_string(), "FrSt");
        assert_eq!(&chunk.data_as_string().unwrap(), "I am the first chunk");
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

    fn testing_png() -> Png {
        let chunks = testing_chunks();

        Png::from_chunks(chunks)
    }
}
