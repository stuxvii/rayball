use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

pub struct BinaryEncoder {
    pub data: Vec<u8>,
    le: bool,
}

macro_rules! w { ($self:ident, $v:expr, $method:ident) => {
    if $self.le { $self.data.$method::<LittleEndian>($v) } 
    else { $self.data.$method::<BigEndian>($v) }.unwrap()
}}

impl BinaryEncoder {
    pub fn new(capacity: usize, le: bool) -> Self {
        Self { data: Vec::with_capacity(capacity), le }
    }
    pub fn append_bytes(&mut self, bytes: &[u8]) { self.data.extend_from_slice(bytes); }

    pub fn w_u8(&mut self, v: u8) { self.data.push(v); }
    pub fn w_i16(&mut self, v: i16) { w!(self, v, write_i16); }
    pub fn w_u16(&mut self, v: u16) { w!(self, v, write_u16); }
    pub fn w_i32(&mut self, v: i32)  { w!(self, v, write_i32); }
    pub fn w_u32(&mut self, v: u32) { w!(self, v, write_u32); }
    pub fn w_f32(&mut self, v: f32) { w!(self, v, write_f32); }
    pub fn w_f64(&mut self, v: f64)  { w!(self, v, write_f64); }

    pub fn w_varint(&mut self, mut v: u32) {
        while v >= 128 {
            self.data.push((v as u8) | 128);
            v >>= 7;
        }
        self.data.push(v as u8);
    }

    fn w_str(&mut self, s: &str) { self.append_bytes(s.as_bytes()); }
    
    pub fn w_str_len(&mut self, s: &str) {
        self.w_varint(s.len() as u32);
        self.w_str(s);
    }

    pub fn w_nullable_str_len(&mut self, s: Option<&str>) {
        match s {
            None => self.w_varint(0),
            Some(str) => { self.w_varint(str.len() as u32 + 1); self.w_str(str); }
        }
    }

    pub fn w_str_limit(&mut self, s: &str) {
        let b: &[u8] = s.as_bytes();
        if b.len() > 255 { panic!("String too long"); }
        self.w_u8(b.len() as u8);
        self.append_bytes(b);
    }

    pub fn w_str_json<T: serde::Serialize>(&mut self, v: &T) {
        self.w_str_len(&serde_json::to_string(v).unwrap());
    }
}

pub struct BinaryDecoder<'a> {
    cursor: Cursor<&'a [u8]>,
    le: bool,
}

macro_rules! r { ($self:ident, $method:ident) => {
    if $self.le { $self.cursor.$method::<LittleEndian>() } 
    else { $self.cursor.$method::<BigEndian>() }.unwrap()
}}

impl<'a> BinaryDecoder<'a> {
    pub fn new(data: &'a [u8], le: bool) -> Self {
        Self { cursor: Cursor::new(data), le }
    }

    pub fn r_i8(&mut self) -> i8 { self.cursor.read_i8().unwrap() }
    pub fn r_u8(&mut self) -> u8 { self.cursor.read_u8().unwrap() }


    pub fn r_i16(&mut self) -> i16 { r!(self, read_i16) }
    pub fn r_u16(&mut self) -> u16 { r!(self, read_u16) }
    pub fn r_i32(&mut self)  -> i32 { r!(self, read_i32) }
    pub fn r_u32(&mut self) -> u32 { r!(self, read_u32) }
    pub fn r_f32(&mut self) -> f32 { r!(self, read_f32) }
    pub fn r_f64(&mut self)  -> f64 { r!(self, read_f64) }

    pub fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf).unwrap();
        buf
    }

    pub fn r_varint(&mut self) -> u32 {
        let mut v = 0u32;
        let mut shift = 0;
        loop {
            let b = self.r_u8();
            v |= ((b & 127) as u32) << shift;
            if b & 128 == 0 { break; }
            shift += 7;
        }
        v
    }

    fn r_utf8(&mut self, len: usize) -> String {
        let bytes = self.read_bytes(len);
        String::from_utf8(bytes).expect("Invalid UTF8")
    }

    pub fn r_str(&mut self) -> String {
        let len: usize = self.r_varint() as usize;
        self.r_utf8(len)
    }

    pub fn r_nul_str(&mut self) -> Option<String> {
        let len: u32 = self.r_varint();
        if len == 0 { None } else { Some(self.r_utf8((len - 1) as usize)) }
    }

    pub fn r_pfx_str(&mut self) -> String {
        let len = self.r_u8() as usize;
        self.r_utf8(len)
    }

    pub fn r_json<T: serde::de::DeserializeOwned>(&mut self) -> T {
        serde_json::from_str(&self.r_str()).unwrap()
    }
}