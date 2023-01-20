use std::{
    collections::HashMap,
    ops::{Add, Sub},
};

pub enum NumType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
}

impl NumType {
    pub fn as_code(self) -> u8 {
        match self {
            NumType::U8 => 0u8,
            NumType::U16 => 1u8,
            NumType::U32 => 2u8,
            NumType::U64 => 3u8,
            NumType::U128 => 4u8,
            NumType::I8 => 5u8,
            NumType::I16 => 6u8,
            NumType::I32 => 7u8,
            NumType::I64 => 8u8,
            NumType::I128 => 9u8,
            NumType::F32 => 10u8,
            NumType::F64 => 11u8,
        }
    }

    pub fn from_code(code: u8) -> Option<NumType> {
        Some(match code {
            0u8 => NumType::U8,
            1u8 => NumType::U16,
            2u8 => NumType::U32,
            3u8 => NumType::U64,
            4u8 => NumType::U128,
            5u8 => NumType::I8,
            6u8 => NumType::I16,
            7u8 => NumType::I32,
            8u8 => NumType::I64,
            9u8 => NumType::I128,
            10u8 => NumType::F32,
            11u8 => NumType::F64,
            _ => return None,
        })
    }
}

pub trait ToBeBytes {
    fn to_be_bytes(&self) -> &[u8; 8];
}
pub trait Num: Add<Output = Self> + Sub<Output = Self> + PartialEq + Copy + ToBeBytes {}

pub struct Metadata(HashMap<String, Vec<u8>>);

impl Metadata {
    pub fn new() -> Self {
        Metadata(HashMap::new())
    }

    pub fn get_num<K>(&self, key: K)
    where
        K: AsRef<str>,
    {
        let num_bytes = self.0.get(key.as_ref()).unwrap();
        let num_type = NumType::from_code(num_bytes[..((u8::BITS / 8) as usize)]);
        let num = &num_bytes[(u8::BITS as usize)..];
    }

    pub fn insert_num<K, N>(&mut self, key: K, num: N, num_type: NumType)
    where
        K: AsRef<str>,
        N: Num,
    {
        self.0.insert(
            key.as_ref().into(),
            [
                num_type.as_code().to_be_bytes().as_slice(),
                num.to_be_bytes().as_slice(),
            ]
            .concat(),
        );
    }

    pub fn insert_str<K, N>(&mut self, key: K, num: N)
    where
        K: AsRef<str>,
        N: AsRef<str>,
    {
        self.0
            .insert(key.as_ref().into(), num.as_ref().as_bytes().to_vec());
    }

    fn as_bytes(self) -> Vec<u8> {
        let bytes = self
            .0
            .iter()
            .map(|(k, v)| {
                let key_bytes = k.as_bytes();
                let key_len_bytes = (key_bytes.len() as u64).to_be_bytes().as_slice();
                let value_len_bytes = (k.len() as u64).to_be_bytes().as_slice();
                [key_len_bytes, key_bytes, value_len_bytes, v].concat()
            })
            .fold(Vec::new(), |acc, v| [acc, v].concat());
        [(bytes.len() as u64).to_be_bytes().as_slice(), &bytes].concat()
    }

    pub fn from_bytes(self) -> Vec<u8> {
        self.0
            .iter()
            .map(|(k, v)| {
                let key_bytes = k.as_bytes();
                let key_len_bytes = &(key_bytes.len() as u64).to_be_bytes()[..];
                let value_len_bytes = &(k.len() as u64).to_be_bytes()[..];
                [key_len_bytes, key_bytes, value_len_bytes, v].concat()
            })
            .fold(Vec::new(), |acc, v| [acc, v].concat())
    }
}
