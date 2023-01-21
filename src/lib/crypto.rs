use aes_gcm::{aead::Aead, aes::cipher::InvalidLength, Aes256Gcm, KeyInit, Nonce};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read, Write},
};

use crate::constants::DEFAULT_CHUNK_LEN;

/// Default initialization vector length
const IV_LEN: usize = 96 / 8; // 12
/// Default authentication tag length
const TAG_LEN: usize = 128 / 8; // 16

#[derive(Debug)]
pub enum ErrorKind {
    InvalidKeyLength(InvalidLength),
    AesError(aes_gcm::Error),
    Io(io::Error),
}

/// A struct for encrypting/decrypting bytes or io streams.
#[derive(Clone)]
pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    /// Creates a new Crypto instance, the given key will be
    /// used for every operation performed.
    pub fn new<K>(key: K) -> Result<Crypto, ErrorKind>
    where
        K: AsRef<[u8]>,
    {
        let key = hash_key(key.as_ref());
        Ok(Crypto {
            cipher: Aes256Gcm::new_from_slice(&key).map_err(|e| ErrorKind::InvalidKeyLength(e))?,
        })
    }

    /// Basic function to encrypt.
    pub fn encrypt_with_iv(&self, plaintext: &[u8], iv: &[u8]) -> Result<Vec<u8>, ErrorKind> {
        Ok(self
            .cipher
            .encrypt(Nonce::from_slice(iv), plaintext)
            .map_err(|e| ErrorKind::AesError(e))?)
    }

    /// Basic function to decrypt.
    pub fn decrypt_with_iv(&self, ciphertext: &[u8], iv: &[u8]) -> Result<Vec<u8>, ErrorKind> {
        Ok(self
            .cipher
            .decrypt(Nonce::from_slice(iv), ciphertext)
            .map_err(|e| ErrorKind::AesError(e))?)
    }

    /// Encrypt a small piece of data.
    ///
    /// Example:
    ///
    /// ```
    /// use fencryption_lib::crypto::Crypto;
    ///
    /// let my_super_key = "this_is_super_secure".as_bytes();
    /// let my_super_secret_message = "hello :)".as_bytes();
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// let enc = crypto.encrypt(my_super_secret_message).unwrap();
    ///
    /// assert_ne!(my_super_secret_message, enc);
    /// ```
    pub fn encrypt<P>(&self, plain: P) -> Result<Vec<u8>, ErrorKind>
    where
        P: AsRef<[u8]>,
    {
        let iv = random_iv();
        Ok([&iv, self.encrypt_with_iv(plain.as_ref(), &iv)?.as_slice()].concat())
    }

    /// Decrypt a small piece of data.
    ///
    /// Example:
    ///
    /// ```
    /// use fencryption_lib::crypto::Crypto;
    ///
    /// let my_super_key = "this_is_super_secure".as_bytes();
    /// let my_super_secret_message = "hello :)".as_bytes();
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// let enc = crypto.encrypt(my_super_secret_message).unwrap();
    /// let dec = crypto.decrypt(&enc).unwrap();
    ///
    /// assert_eq!(my_super_secret_message, dec);
    /// ```
    pub fn decrypt<E>(&self, enc: E) -> Result<Vec<u8>, ErrorKind>
    where
        E: AsRef<[u8]>,
    {
        let iv = &enc.as_ref()[..IV_LEN];
        let ciphertext = &enc.as_ref()[IV_LEN..];

        self.decrypt_with_iv(ciphertext, iv)
    }

    /// Encrypt a stream from a source and a destination
    /// (both [`fs::File`][std::fs::File]).
    ///
    /// Example:
    ///
    /// (See [`TmpDir`][crate::tmp::TmpDir])
    ///
    /// ```rust
    /// use fencryption_lib::crypto::Crypto;
    /// use fencryption_lib::tmp::TmpDir;
    ///
    /// let my_super_key = b"this_is_super_secure";
    /// let my_super_secret_message = b"hello :)";
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// // Creates a temporary directory
    /// let tmp_dir = TmpDir::new().unwrap();
    ///
    /// // tmp_dir.write_file is akin to std::fs::write
    /// tmp_dir
    ///     .write_file("plain", my_super_secret_message)
    ///     .unwrap();
    /// crypto
    ///     .encrypt_stream(
    ///         // tmp_dir.open_file is akin to std::fs::File::open
    ///         &mut tmp_dir.open_file("plain").unwrap(),
    ///         // tmp_dir.create_file is akin to std::fs::File::create
    ///         &mut tmp_dir.create_file("enc").unwrap(),
    ///     )
    ///     .unwrap();
    /// ```
    pub fn encrypt_stream(&self, source: &mut File, dest: &mut File) -> Result<(), ErrorKind> {
        const CHUNK_LEN: usize = DEFAULT_CHUNK_LEN;
        let mut buffer = [0u8; CHUNK_LEN];

        loop {
            let read_len = source.read(&mut buffer).map_err(|e| ErrorKind::Io(e))?;
            dest.write(&self.encrypt(&buffer[..read_len])?)
                .map_err(|e| ErrorKind::Io(e))?;
            // Stops when the loop reached the end of the file
            if read_len != CHUNK_LEN {
                break;
            }
        }

        Ok(())
    }

    /// Decrypt a stream from a source and a destination
    /// (both [`fs::File`][std::fs::File]).
    ///
    /// Example:
    ///
    /// (See [`TmpDir`][crate::tmp::TmpDir])
    ///
    /// ```rust
    /// use fencryption_lib::crypto::Crypto;
    /// use fencryption_lib::tmp::TmpDir;
    ///
    /// let my_super_key = b"this_is_super_secure";
    /// let my_super_secret_message = b"hello :)";
    ///
    /// let crypto = Crypto::new(my_super_key).unwrap();
    ///
    /// // Creates a temporary directory
    /// let tmp_dir = TmpDir::new().unwrap();
    ///
    /// // tmp_dir.write_file is akin to std::fs::write
    /// tmp_dir
    ///     .write_file("plain", my_super_secret_message)
    ///     .unwrap();
    /// crypto
    ///     .encrypt_stream(
    ///         // tmp_dir.open_file is akin to std::fs::File::open
    ///         &mut tmp_dir.open_file("plain").unwrap(),
    ///         // tmp_dir.create_file is akin to std::fs::File::create
    ///         &mut tmp_dir.create_file("enc").unwrap(),
    ///     )
    ///     .unwrap();
    ///
    /// crypto
    ///     .decrypt_stream(
    ///         // tmp_dir.open_file is akin to std::fs::File::open
    ///         &mut tmp_dir.open_file("enc").unwrap(),
    ///         // tmp_dir.create_file is akin to std::fs::File::create
    ///         &mut tmp_dir.create_file("dec").unwrap(),
    ///     )
    ///     .unwrap();
    ///
    /// assert_eq!(tmp_dir.read_file("dec").unwrap(), my_super_secret_message);
    /// ```
    pub fn decrypt_stream(&self, source: &mut File, dest: &mut File) -> Result<(), ErrorKind> {
        const CHUNK_LEN: usize = IV_LEN + DEFAULT_CHUNK_LEN + TAG_LEN; // ciphertext (500) + auth tag (16)
        let mut buffer = [0u8; CHUNK_LEN];

        // let file_len = source.metadata().map_err(|e| ErrorKind::Io(e))?;
        loop {
            let read_len = source.read(&mut buffer).map_err(|e| ErrorKind::Io(e))?;
            dest.write(&self.decrypt(&buffer[..read_len])?)
                .map_err(|e| ErrorKind::Io(e))?;
            // Stops when the loop reached the end of the file.
            if read_len != CHUNK_LEN {
                break;
            }
        }

        Ok(())
    }
}

fn hash_key<K>(key: K) -> Vec<u8>
where
    K: AsRef<[u8]>,
{
    let mut hasher = Sha256::new();
    hasher.update(key.as_ref());
    hasher.finalize().to_vec()
}

fn random_iv() -> Vec<u8> {
    let mut iv = [0; IV_LEN];
    OsRng.fill_bytes(&mut iv);
    iv.to_vec()
}
