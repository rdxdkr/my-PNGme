use crc::{Crc, CRC_32_ISO_HDLC};

use crate::chunk_type::ChunkType;

struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
}

impl Chunk {
    fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        Self { chunk_type, data }
    }

    fn length(&self) -> u32 {
        self.data.len() as u32
    }

    fn crc(&self) -> u32 {
        /*
            from http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html#Chunk-layout

            the crc is calculated on the bytes of the chunk type and data, and it needs to be 4 bytes long

            I had to try out pretty much all of the 32 bit algorithms available in the crc crate, until I found the one that works with the provided test
        */
        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);

        /*
            imperative way by manually iterating over the two sequences
        */
        /*let mut bytes = Vec::<u8>::new();

        for b in self.chunk_type.bytes() {
            bytes.push(b);
        }

        for b in &self.data {
            bytes.push(*b);
        }*/

        /*
            functional way by chaining the two iterators together and collecting them in a new Vec at the end
        */
        let chunk = self.chunk_type.bytes();
        let data = self.data.iter().cloned();

        crc.checksum(&(chunk.iter().cloned().chain(data).collect::<Vec<u8>>()))
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
}
