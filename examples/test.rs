use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

use fencryption_lib::{
    crypto::{DecryptCipher, EncryptCipher},
    io::DEFAULT_BUF_LEN,
};

fn main() {
    let source = OpenOptions::new()
        .read(true)
        .open("_keqing_46k.mp4")
        .unwrap();

    let mut dest = OpenOptions::new()
        .create(true)
        .write(true)
        .open("_keqing_46k.mp4.enc")
        .unwrap();

    let mut cipher = EncryptCipher::new("hello", source);

    let mut buf = [0u8; DEFAULT_BUF_LEN];

    loop {
        let read_len = cipher.read(&mut buf).unwrap();
        dest.write_all(&buf).unwrap();
        if read_len == 0 {
            break;
        }
    }

    let source = OpenOptions::new()
        .read(true)
        .open("_keqing_46k.mp4.enc")
        .unwrap();

    let mut dest = OpenOptions::new()
        .create(true)
        .write(true)
        .open("_keqing_46k.mp4.dec")
        .unwrap();

    let mut cipher = DecryptCipher::new("hello", source);

    let mut buf = [0u8; DEFAULT_BUF_LEN];

    loop {
        let read_len = cipher.read(&mut buf).unwrap();
        dest.write_all(&buf).unwrap();
        if read_len == 0 {
            break;
        }
    }
}
