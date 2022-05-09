use crate::chunk_type::ChunkType;

struct Chunk {}

impl Chunk {
    fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        todo!()
    }

    fn length(&self) -> u32 {
        todo!()
    }

    fn crc(&self) -> u32 {
        todo!()
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
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }
}
