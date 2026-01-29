use byteorder::{BigEndian, ByteOrder, LittleEndian};
use core::panic;
use std::str;

#[derive(Debug, Clone)]
pub struct Encoder {
    data: Vec<u8>,
    le: bool, // true = little endian
    pos: usize,
}

impl Encoder {
    pub fn new(capacity: Option<usize>, le: Option<bool>) -> Self {
        let cap = capacity.unwrap_or(16);
        let le = le.unwrap_or(false);
        Self {
            data: vec![0u8; cap],
            le,
            pos: 0,
        }
    }
    fn ensure_capacity(&mut self, needed: usize) {
        if self.data.len() < needed {
            let new_len = (self.data.len() * 2).max(needed).max(1);
            self.data.resize(new_len, 0);
        }
    }
    pub fn as_slice(&self) -> Vec<u8> {
        self.data[..self.pos].to_vec()
    }
    pub fn buffer_copy(&self) -> &[u8] {
        &self.data[..self.pos]
    }
    pub fn write_u8(&mut self, v: u8) {
        let p: usize = self.pos;
        self.pos += 1;
        self.ensure_capacity(self.pos);
        self.data[p] = v;
    }
    pub fn write_i16(&mut self, v: i16) {
        let p: usize = self.pos;
        self.pos += 2;
        self.ensure_capacity(self.pos);
        if self.le {
            LittleEndian::write_i16(&mut self.data[p..p + 2], v);
        } else {
            BigEndian::write_i16(&mut self.data[p..p + 2], v);
        }
    }
    pub fn write_u16(&mut self, v: u16) {
        let p: usize = self.pos;
        self.pos += 2;
        self.ensure_capacity(self.pos);
        if self.le {
            LittleEndian::write_u16(&mut self.data[p..p + 2], v);
        } else {
            BigEndian::write_u16(&mut self.data[p..p + 2], v);
        }
    }
    pub fn write_i32(&mut self, v: i32) {
        let p = self.pos;
        self.pos += 4;
        self.ensure_capacity(self.pos);
        if self.le {
            LittleEndian::write_i32(&mut self.data[p..p + 4], v);
        } else {
            BigEndian::write_i32(&mut self.data[p..p + 4], v);
        }
    }
    pub fn write_u32(&mut self, v: u32) {
        let p = self.pos;
        self.pos += 4;
        self.ensure_capacity(self.pos);
        if self.le {
            LittleEndian::write_u32(&mut self.data[p..p + 4], v);
        } else {
            BigEndian::write_u32(&mut self.data[p..p + 4], v);
        }
    }
    pub fn write_f32(&mut self, v: f32) {
        let p: usize = self.pos;
        self.pos += 4;
        self.ensure_capacity(self.pos);
        if self.le {
            LittleEndian::write_f32(&mut self.data[p..p + 4], v);
        } else {
            BigEndian::write_f32(&mut self.data[p..p + 4], v);
        }
    }
    pub fn write_f64(&mut self, v: f64) {
        let p = self.pos;
        self.pos += 8;
        self.ensure_capacity(self.pos);
        if self.le {
            LittleEndian::write_f64(&mut self.data[p..p + 8], v);
        } else {
            BigEndian::write_f64(&mut self.data[p..p + 8], v);
        }
    }
    pub fn extend_from_slice(&mut self, bytes: &[u8]) {
        let p: usize = self.pos;
        self.pos += bytes.len();
        self.ensure_capacity(self.pos);
        self.data[p..p + bytes.len()].copy_from_slice(bytes);
    }
    pub fn write_len_prefixed_str(&mut self, s: &str) {
        let bytes: &[u8] = s.as_bytes();
        if bytes.len() > 255 {
            panic!("String too long");
        }
        self.write_u8(bytes.len() as u8);
        self.extend_from_slice(bytes);
    }
    pub fn utf8_bit_length(s: &str) -> usize {
        s.chars().map(|c| c.len_utf8()).sum()
    }
    pub fn leb128_size(a: u32) -> usize {
        if a < 128 {
            1
        } else if a < 16_384 {
            2
        } else if a < 2_097_152 {
            3
        } else if a < 268_435_456 {
            4
        } else if a < 67_108_864 {
            5
        } else if a < 2_147_483_648 {
            6
        } else {
            panic!("Charcode {a} was WAY too big! Report this bug! Program has halted.");
        }
    }
    pub fn write_u32_leb128(&mut self, mut a: u32) {
        let p: usize = self.pos;
        let len: usize = Self::leb128_size(a);
        self.ensure_capacity(p + len);

        let mut i: usize = 0;
        while a >= 0x80 {
            self.data[p + i] = (a as u8) | 0x80;
            a >>= 7;
            i += 1;
        }
        self.data[p + i] = a as u8;
        self.pos += i + 1;
    }
    pub fn write_str_len_leb128(&mut self, s: &str) {
        let len: usize = Self::utf8_bit_length(s);
        self.write_u32_leb128(len as u32);
        self.extend_from_string(s);
    }
    pub fn write_opt_str_len_leb128(&mut self, s: Option<&str>) {
        match s {
            None => self.write_u32_leb128(0),
            Some(v) => {
                let len: usize = Self::utf8_bit_length(v) + 1;
                self.write_u32_leb128(len as u32);
                self.extend_from_string(v);
            }
        }
    }
    fn extend_from_string(&mut self, s: &str) {
        let bytes: &[u8] = s.as_bytes();
        self.ensure_capacity(self.pos + bytes.len());
        self.extend_from_slice(bytes);
    }
}

#[derive(Debug, Clone)]
pub struct Decoder {
    data: Vec<u8>,
    le: bool, // true = little endian
    pos: usize,
}

impl Decoder {
    pub fn new(capacity: Option<usize>, le: Option<bool>) -> Self {
        let cap = capacity.unwrap_or(16);
        let le = le.unwrap_or(false);
        Self {
            data: vec![0u8; cap],
            le,
            pos: 0,
        }
    }
    pub fn read_utf8_char(bytes: &[u8], pos: usize) -> (char, usize) {
        let slice: &[u8] = &bytes[pos..];
        let s: &str = std::str::from_utf8(slice).expect("invalid UTF‑8");
        let ch: char = s.chars().next().unwrap();
        (ch, ch.len_utf8())
    }
    pub fn read_utf8_string_from_length(&mut self, len_bytes: usize) -> Result<String, String> {
        let start: usize = self.pos;
        let end: usize = start + len_bytes;

        if end > self.data.len() {
            return Err(format!(
                "Requested {} bytes from position {}, but only {} bytes remain",
                len_bytes,
                start,
                self.data.len() - start
            ));
        }

        match str::from_utf8(&self.data[start..end]) {
            Ok(slice) => {
                self.pos = end;
                Ok(slice.to_owned())
            }
            Err(e) => Err(format!(
                "Invalid UTF‑8 sequence at position {}: {}",
                start + e.valid_up_to(),
                e
            )),
        }
    }
    pub fn read_leb128(&mut self) -> usize {
        let start: usize = self.pos;
        let mut value: u64 = 0;
        let mut shift: usize = 0usize;

        loop {
            let byte = self.data[start + shift];
            value |= ((byte & 0x7F) as u64) << (shift * 7);

            if (byte & 0x80) == 0 {
                break;
            }

            shift += 1;
            assert!(shift < 5, "LEB128 too long");
        }

        self.pos += shift + 1;
        value as usize
    }
    pub fn read_bytes_from_length(&mut self, mut len: Option<usize>) -> Result<&[u8], String> {
        if len.is_none() { len = Some(self.data.len() - self.pos) }
        if self.pos + len.unwrap() > self.data.len() {
            return Err(String::from("Read too much!"));
        }
        let bytes: &[u8] = &self.data[self.pos..self.pos + len.unwrap()];
        self.pos += len.unwrap();
        Ok(bytes)
    }
    pub fn read_as_slice_from_length(&mut self, len: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let slice: &[u8] = self.read_bytes_from_length(Some(len))?;
        Ok(slice.to_vec())
    }
    pub fn read_string(mut self) -> Result<Option<String>, Box<dyn std::error::Error + 'static>> {
        let a: usize = self.read_leb128();
        if a > 0 {
            Ok(Some(self.read_utf8_string_from_length((a - 1).try_into().unwrap())?))
        } else {
            Ok(None)
        }
    }
    pub fn read_i8_byte(&mut self) -> i8 {
        let b: i8 = self.data[self.pos] as i8;
        self.pos += 1;
        b
    }
    pub fn read_u8_byte(&mut self) -> u8 {
        let b = self.data[self.pos];
        self.pos += 1;
        b
    }
    pub fn read_i16_byte(&mut self) -> i16 {
        if self.le {
            let byte: i16 = LittleEndian::read_i16(&self.data[self.pos..self.pos+2]);
            self.pos += 2;
            return byte
        } else {
            let byte: i16 = BigEndian::read_i16(&self.data[self.pos..self.pos+2]);
            self.pos += 2;
            return byte
        }
    }
    pub fn read_u16_byte(&mut self) -> u16 {
        if self.le {
            let byte: u16 = LittleEndian::read_u16(&self.data[self.pos..self.pos+2]);
            self.pos += 2;
            return byte
        } else {
            let byte: u16 = BigEndian::read_u16(&self.data[self.pos..self.pos+2]);
            self.pos += 2;
            return byte
        }
    }
    pub fn read_i32_byte(&mut self) -> i32 {
        if self.le {
            let byte: i32 = LittleEndian::read_i32(&self.data[self.pos..self.pos+4]);
            self.pos += 4;
            return byte
        } else {
            let byte: i32 = BigEndian::read_i32(&self.data[self.pos..self.pos+4]);
            self.pos += 4;
            return byte
        }
    }
    pub fn read_u32_byte(&mut self) -> u32 {
        if self.le {
            let byte: u32 = LittleEndian::read_u32(&self.data[self.pos..self.pos+4]);
            self.pos += 4;
            return byte
        } else {
            let byte: u32 = BigEndian::read_u32(&self.data[self.pos..self.pos+4]);
            self.pos += 4;
            return byte
        }
    }
    pub fn read_f32_byte(&mut self) -> f32 {
        if self.le {
            let byte: f32 = LittleEndian::read_f32(&self.data[self.pos..self.pos+4]);
            self.pos += 4;
            return byte
        } else {
            let byte: f32 = BigEndian::read_f32(&self.data[self.pos..self.pos+4]);
            self.pos += 4;
            return byte
        }
    }
    pub fn read_f64_byte(&mut self) -> f64 {
        if self.le {
            let byte: f64 = LittleEndian::read_f64(&self.data[self.pos..self.pos+8]);
            self.pos += 8;
            return byte
        } else {
            let byte: f64 = BigEndian::read_f64(&self.data[self.pos..self.pos+8]);
            self.pos += 8;
            return byte
        }
    }
    pub fn read_string_leb128(&mut self) -> Result<String, String> {
        let a = self.read_u8_byte();
        self.read_utf8_string_from_length(a.into())
    }
    pub fn read_string_u8(&mut self) -> Result<String, String> {
        let a = self.read_leb128();
        self.read_utf8_string_from_length(a)
    }
    pub fn jg(&mut self) -> Result<serde_json::Value, String> {
        let a: Result<String, String> = self.read_string_leb128();
        match a {
            Ok(s) => {
                Ok(serde_json::from_str(&s).unwrap())
            },
            Err(e) => Err(e.to_string())
        }
    }
}
