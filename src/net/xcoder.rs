use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

pub struct BinaryEncoder {
    pub data: Vec<u8>,
    le: bool,
}

impl BinaryEncoder {
    pub fn new(le: bool) -> Self {
        Self {
            data: Vec::new(),
            le,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn w_u8(&mut self, v: u8) {
        self.data.push(v);
    }

    pub fn w_i16(&mut self, v: i16) {
        if self.le {
            self.data.write_i16::<LittleEndian>(v).unwrap();
        } else {
            self.data.write_i16::<BigEndian>(v).unwrap();
        }
    }

    pub fn w_u16(&mut self, v: u16) {
        if self.le {
            self.data.write_u16::<LittleEndian>(v).unwrap();
        } else {
            self.data.write_u16::<BigEndian>(v).unwrap();
        }
    }

    pub fn w_i32(&mut self, v: i32) {
        if self.le {
            self.data.write_i32::<LittleEndian>(v).unwrap();
        } else {
            self.data.write_i32::<BigEndian>(v).unwrap();
        }
    }

    pub fn w_u32(&mut self, v: u32) {
        if self.le {
            self.data.write_u32::<LittleEndian>(v).unwrap();
        } else {
            self.data.write_u32::<BigEndian>(v).unwrap();
        }
    }

    pub fn w_f32(&mut self, v: f32) {
        if self.le {
            self.data.write_f32::<LittleEndian>(v).unwrap();
        } else {
            self.data.write_f32::<BigEndian>(v).unwrap();
        }
    }

    pub fn w_f64(&mut self, v: f64) {
        if self.le {
            self.data.write_f64::<LittleEndian>(v).unwrap();
        } else {
            self.data.write_f64::<BigEndian>(v).unwrap();
        }
    }

    pub fn append_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn w_varint(&mut self, mut v: u32) {
        loop {
            let mut byte = (v & 0x7F) as u8;
            v >>= 7;
            if v != 0 {
                byte |= 0x80;
                self.w_u8(byte);
            } else {
                self.w_u8(byte);
                break;
            }
        }
    }

    pub fn w_str(&mut self, s: &str) {
        self.w_varint(s.len() as u32);
        self.append_bytes(s.as_bytes());
    }

    pub fn w_nullable_str(&mut self, s: Option<&str>) {
        match s {
            None => self.w_varint(0),
            Some(str) => {
                self.w_varint(str.len() as u32 + 1);
                self.append_bytes(str.as_bytes());
            }
        }
    }

    pub fn w_small_str(&mut self, s: &str) {
        let b = s.as_bytes();
        if b.len() > 255 {
            panic!("String too long for w_small_str (limit 255)");
        }
        self.w_u8(b.len() as u8);
        self.append_bytes(b);
    }

    pub fn w_json<T: serde::Serialize>(&mut self, v: &T) {
        let s = serde_json::to_string(v).expect("Failed to serialize JSON");
        self.w_str(&s);
    }
}

pub struct BinaryDecoder<'a> {
    cursor: Cursor<&'a [u8]>,
    le: bool,
}

impl<'a> BinaryDecoder<'a> {
    pub fn new(data: &'a [u8], le: bool) -> Self {
        Self {
            cursor: Cursor::new(data),
            le,
        }
    }

    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn r_i8(&mut self) -> i8 {
        self.cursor.read_i8().unwrap()
    }

    pub fn r_u8(&mut self) -> u8 {
        self.cursor.read_u8().unwrap()
    }

    pub fn r_i16(&mut self) -> i16 {
        if self.le {
            self.cursor.read_i16::<LittleEndian>().unwrap()
        } else {
            self.cursor.read_i16::<BigEndian>().unwrap()
        }
    }

    pub fn r_u16(&mut self) -> u16 {
        if self.le {
            self.cursor.read_u16::<LittleEndian>().unwrap()
        } else {
            self.cursor.read_u16::<BigEndian>().unwrap()
        }
    }

    pub fn r_i32(&mut self) -> i32 {
        if self.le {
            self.cursor.read_i32::<LittleEndian>().unwrap()
        } else {
            self.cursor.read_i32::<BigEndian>().unwrap()
        }
    }

    pub fn r_u32(&mut self) -> u32 {
        if self.le {
            self.cursor.read_u32::<LittleEndian>().unwrap()
        } else {
            self.cursor.read_u32::<BigEndian>().unwrap()
        }
    }

    pub fn r_f32(&mut self) -> f32 {
        if self.le {
            self.cursor.read_f32::<LittleEndian>().unwrap()
        } else {
            self.cursor.read_f32::<BigEndian>().unwrap()
        }
    }

    pub fn r_f64(&mut self) -> f64 {
        if self.le {
            self.cursor.read_f64::<LittleEndian>().unwrap()
        } else {
            self.cursor.read_f64::<BigEndian>().unwrap()
        }
    }

    pub fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf).unwrap();
        buf
    }

    pub fn r_bytes_len(&mut self, len: usize) -> Vec<u8> {
        self.read_bytes(len)
    }

    pub fn r_varint(&mut self) -> u32 {
        let mut value = 0u32;
        let mut shift = 0;
        let mut byte: u8;
        loop {
            byte = self.r_u8();
            value |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift >= 35 {
                panic!("NEEDY VARINT OVERFLOW");
            }
        }
        value
    }

    fn read_string_utf8(&mut self, len: usize) -> String {
        let bytes = self.read_bytes(len);
        String::from_utf8(bytes).expect("Invalid UTF-8 string")
    }

    pub fn r_str(&mut self) -> String {
        let len = self.r_varint() as usize;
        self.read_string_utf8(len)
    }

    pub fn r_nullable_str(&mut self) -> Option<String> {
        let len = self.r_varint();
        if len == 0 {
            None
        } else {
            Some(self.read_string_utf8((len - 1) as usize))
        }
    }

    pub fn r_small_str(&mut self) -> String {
        let len = self.r_u8() as usize;
        self.read_string_utf8(len)
    }

    pub fn r_json<T: serde::de::DeserializeOwned>(&mut self) -> T {
        let s = self.r_str();
        serde_json::from_str(&s).unwrap()
    }

    pub fn set_position(&mut self, pos: u64) {
        self.cursor.set_position(pos);
    }
}