use std::{
    collections::HashMap,
    ops::{Add, Sub},
};

pub struct Metadata(HashMap<String, Vec<u8>>);

pub trait ToBeBytes {
    fn to_be_bytes(&self) -> &[u8; 8];
}
pub trait Num: Add<Output = Self> + Sub<Output = Self> + PartialEq + Copy + ToBeBytes {}

impl Metadata {
    pub fn new() -> Self {
        Metadata(HashMap::new())
    }

    pub fn insert_num<K, N>(&mut self, key: K, num: N)
    where
        K: AsRef<str>,
        N: Num,
    {
        self.0
            .insert(key.as_ref().into(), num.to_be_bytes().to_vec());
    }

    pub fn as_bytes(self) -> Vec<u8> {
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
