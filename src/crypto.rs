use aes_gcm::{aead::OsRng, AeadInPlace, Aes256Gcm, KeyInit, Nonce};
use anyhow::anyhow;
use rand::RngCore;
use ring::digest;

const NONCE_LEN: usize = 96 / 8;
const TAG_LEN: usize = 128 / 8;

pub struct Crypto {
    hashed_key: Vec<u8>,
}

impl Crypto {
    pub fn new(key: Vec<u8>) -> Crypto {
        Crypto {
            hashed_key: Self::hash_key(key.as_slice()),
        }
    }

    pub fn encrypt(&self, plain_data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = match Aes256Gcm::new_from_slice(&self.hashed_key) {
            Ok(v) => v,
            Err(e) => return Err(anyhow!(e)),
        };

        let nonce = Nonce::from(Self::generate_nonce());
        let mut buffer = Vec::with_capacity(plain_data.len() + TAG_LEN);
        buffer.extend_from_slice(plain_data);

        println!("buffer: {:x?}", &buffer);

        if let Err(e) = cipher.encrypt_in_place(&nonce, &[], &mut buffer) {
            return Err(anyhow!(e));
        };

        let result = [nonce.to_vec(), buffer].concat();

        println!("nonce: {:x?}", &nonce);
        println!("result: {:x?}", &result);

        Ok(result)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = match Aes256Gcm::new_from_slice(&self.hashed_key) {
            Ok(v) => v,
            Err(e) => return Err(anyhow!(e)),
        };

        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_LEN]);
        let mut buffer = encrypted_data[NONCE_LEN..].to_vec();

        println!("nonce: {:x?}", &nonce);
        println!("full enc data: {:x?}", &encrypted_data);

        if let Err(e) = cipher.decrypt_in_place(nonce, &[], &mut buffer) {
            return Err(anyhow!(e));
        };

        Ok(buffer)
    }

    fn hash_key(key: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA256, key).as_ref().to_owned()
    }

    fn generate_nonce() -> [u8; NONCE_LEN] {
        let mut nonce = [0; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }
}
