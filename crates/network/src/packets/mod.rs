pub mod clientbound;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::Serialize;
use std::io::{self, Cursor, Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct SlotData {
    pub item_id: i32,
    pub item_count: i8,
    pub nbt: Option<nbt::Blob>,
}

#[derive(Debug)]
pub struct PalettedContainer {
    pub bits_per_entry: u8,
    pub palette: Option<Vec<i32>>,
    pub data_array: Vec<u64>,
}

pub type DecodeResult<T> = std::result::Result<T, PacketDecodeError>;

#[derive(Debug)]
pub enum PacketDecodeError {
    Io(io::Error),
    FromUtf8(std::string::FromUtf8Error),
    Nbt(nbt::Error),
}

impl From<nbt::Error> for PacketDecodeError {
    fn from(err: nbt::Error) -> PacketDecodeError {
        PacketDecodeError::Nbt(err)
    }
}

impl From<io::Error> for PacketDecodeError {
    fn from(err: io::Error) -> PacketDecodeError {
        PacketDecodeError::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for PacketDecodeError {
    fn from(err: std::string::FromUtf8Error) -> PacketDecodeError {
        PacketDecodeError::FromUtf8(err)
    }
}

#[derive(Debug)]
pub enum PacketEncodeError {}

impl<T: std::convert::AsRef<[u8]>> PacketDecoderExt for Cursor<T> {}
impl PacketDecoderExt for TcpStream {}

pub trait PacketDecoderExt: Read + Sized {
    fn read_unsigned_byte(&mut self) -> DecodeResult<u8> {
        Ok(self.read_u8()?)
    }

    fn read_byte(&mut self) -> DecodeResult<i8> {
        Ok(self.read_i8()?)
    }

    fn read_bytes(&mut self, bytes: usize) -> DecodeResult<Vec<u8>> {
        let mut read = vec![0; bytes];
        self.read_exact(&mut read)?;
        Ok(read)
    }

    fn read_long(&mut self) -> DecodeResult<i64> {
        Ok(self.read_i64::<BigEndian>()?)
    }

    fn read_int(&mut self) -> DecodeResult<i32> {
        Ok(self.read_i32::<BigEndian>()?)
    }

    fn read_short(&mut self) -> DecodeResult<i16> {
        Ok(self.read_i16::<BigEndian>()?)
    }

    fn read_unsigned_short(&mut self) -> DecodeResult<u16> {
        Ok(self.read_u16::<BigEndian>()?)
    }

    fn read_double(&mut self) -> DecodeResult<f64> {
        Ok(self.read_f64::<BigEndian>()?)
    }

    fn read_float(&mut self) -> DecodeResult<f32> {
        Ok(self.read_f32::<BigEndian>()?)
    }

    fn read_bool(&mut self) -> DecodeResult<bool> {
        Ok(self.read_u8()? == 1)
    }

    fn read_varint(&mut self) -> DecodeResult<i32> {
        let mut num_read = 0;
        let mut result = 0i32;
        let mut read;
        loop {
            read = self.read_byte()? as u8;
            let value = (read & 0b0111_1111) as i32;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                panic!("VarInt is too big!");
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(result)
    }

    fn read_varlong(&mut self) -> DecodeResult<i64> {
        let mut num_read = 0;
        let mut result = 0i64;
        let mut read;
        loop {
            read = self.read_byte()? as u8;
            let value = (read & 0b0111_1111) as i64;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                panic!("VarInt is too big!");
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(result)
    }

    fn read_string(&mut self) -> DecodeResult<String> {
        let length = self.read_varint()?;
        Ok(String::from_utf8(self.read_bytes(length as usize)?)?)
    }

    fn read_to_end(&mut self) -> DecodeResult<Vec<u8>> {
        let mut data = Vec::new();
        let _ = Read::read_to_end(self, &mut data);
        Ok(data)
    }

    fn read_position(&mut self) -> DecodeResult<(i32, i32, i32)> {
        let val: i64 = self.read_long()?;
        let x = val >> 38;
        let mut y = val & 0xFFF;
        if y >= 0x800 {
            y -= 0x1000;
        }
        let z = val << 26 >> 38;
        Ok((x as i32, y as i32, z as i32))
    }

    fn read_nbt_blob(&mut self) -> DecodeResult<Option<nbt::Blob>> {
        match nbt::Blob::from_reader(self) {
            Ok(nbt) => Ok(Some(nbt)),
            Err(nbt::Error::NoRootCompound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

pub trait PacketEncoderExt: Write {
    fn write_boolean(&mut self, val: bool) {
        self.write_all(&[val as u8]).unwrap();
    }
    fn write_bytes(&mut self, val: &[u8]) {
        self.write_all(val).unwrap();
    }
    fn write_varint(&mut self, val: i32) {
        let _ = self.write_all(&PacketEncoder::varint(val));
    }

    fn write_varlong(&mut self, mut val: i64) {
        loop {
            let mut temp = (val & 0b1111_1111) as u8;
            val >>= 7;
            if val != 0 {
                temp |= 0b1000_0000;
            }
            self.write_all(&[temp]).unwrap();
            if val == 0 {
                break;
            }
        }
    }

    fn write_byte(&mut self, val: i8) {
        self.write_all(&[val as u8]).unwrap();
    }

    fn write_unsigned_byte(&mut self, val: u8) {
        self.write_all(&[val]).unwrap();
    }

    fn write_short(&mut self, val: i16) {
        self.write_i16::<BigEndian>(val).unwrap();
    }

    fn write_unsigned_short(&mut self, val: u16) {
        self.write_u16::<BigEndian>(val).unwrap();
    }

    fn write_int(&mut self, val: i32) {
        self.write_i32::<BigEndian>(val).unwrap();
    }

    fn write_double(&mut self, val: f64) {
        self.write_f64::<BigEndian>(val).unwrap();
    }

    fn write_float(&mut self, val: f32) {
        self.write_f32::<BigEndian>(val).unwrap();
    }

    fn write_string(&mut self, n: usize, val: &str) {
        if val.len() > n * 4 + 3 {
            panic!("Tried to write string longer than the max length!");
        }
        self.write_varint(val.len() as i32);
        self.write_all(val.as_bytes()).unwrap();
    }

    fn write_uuid(&mut self, val: u128) {
        self.write_u128::<BigEndian>(val).unwrap();
    }

    fn write_long(&mut self, val: i64) {
        self.write_i64::<BigEndian>(val).unwrap();
    }

    fn write_position(&mut self, x: i32, y: i32, z: i32) {
        let long =
            ((x as i64 & 0x3FF_FFFF) << 38) | ((z as i64 & 0x3FF_FFFF) << 12) | (y as i64 & 0xFFF);
        self.write_long(long);
    }

    fn write_bool(&mut self, val: bool) {
        self.write_u8(val as u8).unwrap();
    }

    fn write_nbt<T: Serialize>(&mut self, nbt: &T) {
        let _ = nbt::to_writer(self, nbt, None);
    }

    fn write_nbt_blob(&mut self, blob: &nbt::Blob)
    where
        Self: Sized,
    {
        blob.to_writer(self).unwrap();
    }

    fn write_slot_data(&mut self, slot_data: &Option<SlotData>)
    where
        Self: Sized,
    {
        if let Some(slot) = slot_data {
            self.write_bool(true);
            self.write_varint(slot.item_id);
            self.write_byte(slot.item_count);
            if let Some(nbt) = &slot.nbt {
                self.write_nbt_blob(nbt);
            } else {
                self.write_byte(0); // End tag
            }
        } else {
            self.write_bool(false);
        }
    }
}

impl PacketEncoderExt for Vec<u8> {}

pub struct PacketEncoder {
    buffer: Vec<u8>,
    packet_id: u32,
    // c_cache: Option<Vec<u8>>,
    // unc_cache: Option<Vec<u8>>,
}

impl PacketEncoder {
    fn new(buffer: Vec<u8>, packet_id: u32) -> PacketEncoder {
        PacketEncoder { buffer, packet_id }
    }

    // This function is seperate because it is needed when writing packet headers
    fn varint(val: i32) -> Vec<u8> {
        let mut val = val as u32;
        let mut buf = Vec::new();
        loop {
            let mut temp = (val & 0b1111_1111) as u8;
            val >>= 7;
            if val != 0 {
                temp |= 0b1000_0000;
            }
            buf.push(temp);
            if val == 0 {
                return buf;
            }
        }
    }

    pub fn write_compressed(&self, mut w: impl Write) -> io::Result<()> {
        // TODO: zero allocation
        let packet_id = PacketEncoder::varint(self.packet_id as i32);
        let data = [packet_id.as_slice(), self.buffer.as_slice()].concat();
        if self.buffer.len() < 256 {
            // Data Length adds another byte
            let packet_length = PacketEncoder::varint((1 + data.len()) as i32);

            w.write_all(&packet_length)?;
            // Data Length: 0 because uncompressed
            w.write_all(&[0])?;
            w.write_all(&data)?;
        } else {
            let data_length = PacketEncoder::varint(data.len() as i32);
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&data)?;
            let compressed = encoder.finish().unwrap();
            let packet_length =
                PacketEncoder::varint((data_length.len() + compressed.len()) as i32);

            w.write_all(&packet_length)?;
            w.write_all(&data_length)?;
            w.write_all(&compressed)?;
        }

        // self.c_cache = Some(finished);
        // return self.c_cache.as_ref().unwrap();

        Ok(())
    }

    pub fn write_uncompressed(&self, mut w: impl Write) -> io::Result<()> {
        // if let Some(data) = &self.unc_cache {
        //     return &data;
        // }

        let packet_id = PacketEncoder::varint(self.packet_id as i32);
        let length = PacketEncoder::varint((self.buffer.len() + packet_id.len()) as i32);

        // https://github.com/rust-lang/rust/issues/70436
        w.write_all(&length)?;
        w.write_all(&packet_id)?;
        w.write_all(&self.buffer)?;

        // self.unc_cache = Some([&length[..], &packet_id[..], &self.buffer[..]].concat());
        // return self.c_cache.as_ref().unwrap();

        Ok(())
    }
}
