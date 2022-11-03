mod util;

use std::{
    env,
    fs::{self, File},
};

use crate::crypto::Crypto;

// Crypto tests

const KEY: &[u8] = b"my_super_key";
const PLAIN_DATA: &[u8] = b"hello :)";

#[test]
fn encrypt_data_without_error() {
    let crypto = Crypto::new(KEY).unwrap();
    crypto.encrypt(PLAIN_DATA).unwrap();
}

#[test]
fn decrypt_data_without_error() {
    let crypto = Crypto::new(KEY).unwrap();
    let encrypted_data = crypto.encrypt(PLAIN_DATA).unwrap();
    crypto.decrypt(&encrypted_data).unwrap();
}

#[test]
fn can_encrypt_then_decrypt_and_get_original_data() {
    let crypto = Crypto::new(KEY).unwrap();
    let encrypted_data = crypto.encrypt(PLAIN_DATA).unwrap();
    let new_plain_data = crypto.decrypt(&encrypted_data).unwrap();
    assert_eq!(&new_plain_data, PLAIN_DATA);
}

#[test]
#[should_panic]
fn fail_to_decrypt_data_when_key_is_wrong() {
    let crypto1 = Crypto::new(KEY).unwrap();
    let crypto2 = Crypto::new(&[KEY, b"nope"].concat()).unwrap();
    let encrypted_data = crypto1.encrypt(PLAIN_DATA).unwrap();
    crypto2.decrypt(&encrypted_data).unwrap();
}

#[test]
fn encrypt_file_without_error() {
    let crypto = Crypto::new(KEY).unwrap();

    let tmp_dir = env::temp_dir();
    let plain_path = tmp_dir.join("encrypt_stream_test");
    let enc_path = tmp_dir.join("encrypt_stream_test.enc");
    let dec_path = tmp_dir.join("encrypt_stream_test.dec");

    fs::remove_file(&plain_path).ok();
    fs::remove_file(&enc_path).ok();
    fs::remove_file(&dec_path).ok();

    fs::write(&plain_path, PLAIN_DATA).unwrap();

    let mut plain = File::open(&plain_path).unwrap();
    let mut enc = File::create(&enc_path).unwrap();

    crypto.encrypt_stream(&mut plain, &mut enc).unwrap();
}

#[test]
fn decrypt_file_without_error() {
    let crypto = Crypto::new(KEY).unwrap();

    let tmp_dir = env::temp_dir();
    let plain_path = tmp_dir.join("encrypt_stream_test");
    let enc_path = tmp_dir.join("encrypt_stream_test.enc");
    let dec_path = tmp_dir.join("encrypt_stream_test.dec");

    fs::remove_file(&plain_path).ok();
    fs::remove_file(&enc_path).ok();
    fs::remove_file(&dec_path).ok();

    fs::write(&plain_path, PLAIN_DATA).unwrap();

    let mut plain = File::open(&plain_path).unwrap();
    let mut enc = File::create(&enc_path).unwrap();

    crypto.encrypt_stream(&mut plain, &mut enc).unwrap();

    let mut enc = File::open(&enc_path).unwrap();
    let mut dec = File::create(&dec_path).unwrap();

    crypto.decrypt_stream(&mut enc, &mut dec).unwrap();
}

#[test]
fn encrypt_then_decrypt_file_and_get_original_data() {
    let crypto = Crypto::new(KEY).unwrap();

    let tmp_dir = env::temp_dir();
    let plain_path = tmp_dir.join("encrypt_stream_test");
    let enc_path = tmp_dir.join("encrypt_stream_test.enc");
    let dec_path = tmp_dir.join("encrypt_stream_test.dec");

    fs::remove_file(&plain_path).ok();
    fs::remove_file(&enc_path).ok();
    fs::remove_file(&dec_path).ok();

    fs::write(&plain_path, PLAIN_DATA).unwrap();

    let mut plain = File::open(&plain_path).unwrap();
    let mut enc = File::create(&enc_path).unwrap();

    crypto.encrypt_stream(&mut plain, &mut enc).unwrap();

    let mut enc = File::open(&enc_path).unwrap();
    let mut dec = File::create(&dec_path).unwrap();

    crypto.decrypt_stream(&mut enc, &mut dec).unwrap();

    assert_eq!(fs::read(&dec_path).unwrap(), PLAIN_DATA[..]);
}

#[test]
#[should_panic]
fn fail_to_decrypt_file_when_key_is_wrong() {
    let crypto1 = Crypto::new(KEY).unwrap();
    let crypto2 = Crypto::new(&[KEY, b"nope"].concat()).unwrap();

    let tmp_dir = env::temp_dir();
    let plain_path = tmp_dir.join("encrypt_stream_test");
    let enc_path = tmp_dir.join("encrypt_stream_test.enc");
    let dec_path = tmp_dir.join("encrypt_stream_test.dec");

    fs::remove_file(&plain_path).ok();
    fs::remove_file(&enc_path).ok();
    fs::remove_file(&dec_path).ok();

    fs::write(&plain_path, PLAIN_DATA).unwrap();

    let mut plain = File::open(&plain_path).unwrap();
    let mut enc = File::create(&enc_path).unwrap();

    crypto1.encrypt_stream(&mut plain, &mut enc).unwrap();

    let mut enc = File::open(&enc_path).unwrap();
    let mut dec = File::create(&dec_path).unwrap();

    crypto2.decrypt_stream(&mut enc, &mut dec).unwrap();

    assert_eq!(PLAIN_DATA[..], fs::read(&dec_path).unwrap());
}

// WalkDir tests
