use std::{str, str::FromStr};

use crate::{
    chunk::{Chunk, InvalidCrcError},
    chunk_type::ChunkType,
    Error, Result,
};

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
            let length = u32::from_be_bytes(value[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;
            let chunk_type = str::from_utf8(&value[cursor..cursor + 4]).unwrap();
            let chunk_type_2 = ChunkType::from_str(chunk_type).unwrap();
            cursor += 4;

            let data_end_index = cursor + length as usize;
            let chunk_data = str::from_utf8(&value[cursor..data_end_index])
                .unwrap()
                .as_bytes()
                .to_vec();
            cursor += length as usize;

            let input_crc = u32::from_be_bytes(
                value[data_end_index..data_end_index + 4]
                    .try_into()
                    .unwrap(),
            );
            cursor += 4;
            let crc = Chunk::calculate_crc(&chunk_type_2, &chunk_data);

            if crc != input_crc {
                return Err(Box::new(InvalidCrcError));
            }

            chunks.push(
                Chunk::try_from(
                    length
                        .to_be_bytes()
                        .iter()
                        .chain(chunk_type.as_bytes().iter())
                        .chain(chunk_data.iter())
                        .chain(crc.to_be_bytes().iter())
                        .copied()
                        .collect::<Vec<u8>>()
                        .as_ref(),
                )
                .unwrap(),
            );
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

    fn testing_chunks() -> Vec<Chunk> {
        let mut chunks = Vec::new();

        chunks.push(chunk_from_strings("FrSt", "I am the first chunk").unwrap());
        chunks.push(chunk_from_strings("miDl", "I am another chunk").unwrap());
        chunks.push(chunk_from_strings("LASt", "I am the last chunk").unwrap());
        chunks
    }

    fn chunk_from_strings(chunk_type: &str, data: &str) -> Result<Chunk> {
        use std::str::FromStr;

        let chunk_type = ChunkType::from_str(chunk_type)?;
        let data: Vec<u8> = data.bytes().collect();

        Ok(Chunk::new(chunk_type, data))
    }
}
