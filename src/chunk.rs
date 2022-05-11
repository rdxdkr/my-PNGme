use crc::{Crc, CRC_32_ISO_HDLC};
use std::str::{self, FromStr};

use crate::{chunk_type::ChunkType, Error};

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

    fn chunk_type(&self) -> &ChunkType {
        todo!()
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        /*
            a slice of u8 (byte) interpreted as a png chunk is structured as follows:
            - first 4 bytes: data length (n)
            - next 4 bytes: chunk type
            - next n bytes: message
            - last 4 bytes: crc
        */

        // the length is encoded as big endian bytes, so it must be read like this
        let data_length = u32::from_be_bytes(value[..4].try_into().unwrap());
        let chunk_type_raw_str = str::from_utf8(&value[4..8]).unwrap();
        let message = str::from_utf8(&value[8..8 + data_length as usize]).unwrap();

        // the crc part is ignored for now, it will be added later as a struct field
        Ok(Chunk {
            chunk_type: ChunkType::from_str(chunk_type_raw_str).unwrap(),
            data: message.as_bytes().to_vec(),
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
