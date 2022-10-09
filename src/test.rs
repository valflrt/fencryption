use std::{
    env,
    fs::{self, File},
    io::Write,
};

use crate::crypto::Crypto;

const KEY: &[u8] = b"my_super_key";
const PLAIN_DATA: &[u8] = b"hello :)";

#[test]
fn encrypt_data_without_error() {
    let crypto = Crypto::new(KEY);

    match crypto.encrypt(PLAIN_DATA) {
        Ok(_) => (),
        Err(_) => panic!("Failed to encrypt"),
    };
}

#[test]
fn decrypt_data_without_error() {
    let crypto = Crypto::new(KEY);

    let encrypted_data = match crypto.encrypt(PLAIN_DATA) {
        Ok(enc) => enc,
        Err(_) => panic!("Failed to encrypt"),
    };

    match crypto.decrypt(&encrypted_data) {
        Ok(_) => (),
        Err(_) => panic!("Failed to decrypt"),
    };
}

#[test]
fn can_encrypt_then_decrypt_and_get_original_data() {
    let crypto = Crypto::new(KEY);

    let encrypted_data = match crypto.encrypt(PLAIN_DATA) {
        Ok(enc) => enc,
        Err(_) => panic!("Failed to encrypt"),
    };

    let new_plain_data = match crypto.decrypt(&encrypted_data) {
        Ok(dec) => dec,
        Err(_) => panic!("Failed to decrypt"),
    };

    assert_eq!(&new_plain_data, PLAIN_DATA);
}

#[test]
#[should_panic]
fn fail_to_decrypt_data_when_key_is_wrong() {
    let crypto1 = Crypto::new(KEY);
    let crypto2 = Crypto::new(&[KEY, b"nope"].concat());

    let encrypted_data = match crypto1.encrypt(PLAIN_DATA) {
        Ok(enc) => enc,
        Err(_) => panic!("Failed to encrypt"),
    };

    match crypto2.decrypt(&encrypted_data) {
        Ok(_) => (),
        Err(_) => panic!("Failed to decrypt"),
    };
}

#[test]
fn encrypt_stream_without_error() {
    let crypto = Crypto::new(KEY);

    File::create(&env::temp_dir().join("encrypt_stream_test"))
        .expect("Failed to create source file")
        .write_all(PLAIN_DATA)
        .expect("Failed to write test data to source file");

    let mut source = File::open(&env::temp_dir().join("encrypt_stream_test"))
        .expect("Failed to open destination file");
    let mut dest = File::create(&env::temp_dir().join("encrypt_stream_test.enc"))
        .expect("Failed to create destination file");

    crypto
        .encrypt_stream(&mut source, &mut dest)
        .expect("Failed to encrypt file");
}

#[test]
fn decrypt_stream_without_error() {
    let crypto = Crypto::new(KEY);

    File::create(&env::temp_dir().join("encrypt_stream_test"))
        .expect("Failed to create source file")
        .write_all(PLAIN_DATA)
        .expect("Failed to write test data to source file");

    let mut source = File::open(&env::temp_dir().join("encrypt_stream_test"))
        .expect("Failed to open destination file");
    let mut dest = File::create(&env::temp_dir().join("encrypt_stream_test.enc"))
        .expect("Failed to create destination file");

    crypto
        .encrypt_stream(&mut source, &mut dest)
        .expect("Failed to encrypt file");

    let mut source = File::open(&env::temp_dir().join("encrypt_stream_test.enc"))
        .expect("Failed to open destination file");
    let mut dest = File::create(&env::temp_dir().join("encrypt_stream_test.dec"))
        .expect("Failed to create destination file");

    crypto
        .decrypt_stream(&mut source, &mut dest)
        .expect("Failed to decrypt file");
}

#[test]
fn encrypt_then_decrypt_stream_and_get_original_data() {
    let crypto = Crypto::new(KEY);

    File::create(&env::temp_dir().join("encrypt_stream_test"))
        .expect("Failed to create source file")
        .write_all(PLAIN_DATA)
        .expect("Failed to write test data to source file");

    let mut source = File::open(&env::temp_dir().join("encrypt_stream_test"))
        .expect("Failed to open destination file");
    let mut dest = File::create(&env::temp_dir().join("encrypt_stream_test.enc"))
        .expect("Failed to create destination file");

    crypto
        .encrypt_stream(&mut source, &mut dest)
        .expect("Failed to encrypt file");

    let mut source = File::open(&env::temp_dir().join("encrypt_stream_test.enc"))
        .expect("Failed to open destination file");
    let mut dest = File::create(&env::temp_dir().join("encrypt_stream_test.dec"))
        .expect("Failed to create destination file");

    crypto
        .decrypt_stream(&mut source, &mut dest)
        .expect("Failed to decrypt file");

    println!(
        "{}",
        fs::read_to_string(&env::temp_dir().join("encrypt_stream_test.dec"))
            .expect("Failed to read decrypted file")
    );

    assert_eq!(
        PLAIN_DATA,
        fs::read(&env::temp_dir().join("encrypt_stream_test.dec"))
            .expect("Failed to read decrypted file")
    )
}

#[test]
#[should_panic]
fn fail_to_decrypt_stream_when_key_is_wrong() {
    let crypto1 = Crypto::new(KEY);
    let crypto2 = Crypto::new(&[KEY, b"nope"].concat());

    File::create(&env::temp_dir().join("encrypt_stream_test"))
        .expect("Failed to create source file")
        .write_all(PLAIN_DATA)
        .expect("Failed to write test data to source file");

    let mut source = File::open(&env::temp_dir().join("encrypt_stream_test"))
        .expect("Failed to open destination file");
    let mut dest = File::create(&env::temp_dir().join("encrypt_stream_test.enc"))
        .expect("Failed to create destination file");

    crypto1
        .encrypt_stream(&mut source, &mut dest)
        .expect("Failed to encrypt file");

    let mut source = File::open(&env::temp_dir().join("encrypt_stream_test.enc"))
        .expect("Failed to open destination file");
    let mut dest = File::create(&env::temp_dir().join("encrypt_stream_test.dec"))
        .expect("Failed to create destination file");

    crypto2
        .decrypt_stream(&mut source, &mut dest)
        .expect("Failed to decrypt file");

    println!(
        "{}",
        fs::read_to_string(&env::temp_dir().join("encrypt_stream_test.dec"))
            .expect("Failed to read decrypted file")
    );

    assert_eq!(
        PLAIN_DATA,
        fs::read(&env::temp_dir().join("encrypt_stream_test.dec"))
            .expect("Failed to read decrypted file")
    )
}
