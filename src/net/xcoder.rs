use byteorder::{ByteOrder, LittleEndian, BigEndian};
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
        let p = self.pos;
        self.pos += 2;
        self.ensure_capacity(self.pos);
        if self.le {
            LittleEndian::write_i16(&mut self.data[p..p + 2], v);
        } else {
            BigEndian::write_i16(&mut self.data[p..p + 2], v);
        }
    }

    pub fn write_u16(&mut self, v: u16) {
        let p = self.pos;
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
        let p = self.pos;
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
        let len = Self::utf8_bit_length(s);
        self.write_u32_leb128(len as u32);
        self.extend_from_string(s);
    }

    pub fn write_opt_str_len_leb128(&mut self, s: Option<&str>) {
        match s {
            None => self.write_u32_leb128(0),
            Some(v) => {
                let len = Self::utf8_bit_length(v) + 1;
                self.write_u32_leb128(len as u32);
                self.extend_from_string(v);
            }
        }
    }

    fn extend_from_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.ensure_capacity(self.pos + bytes.len());
        self.extend_from_slice(bytes);
    }
}