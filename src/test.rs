use crate::crypto::Crypto;

const KEY: &str = "my_super_key";
const PLAIN_DATA: &str = "hello :)";

#[test]
fn encrypt_data_without_error() {
    let crypto = match Crypto::new(KEY.as_bytes()) {
        Ok(v) => v,
        Err(e) => panic!("Failed to create crypto instance: {}", e),
    };

    match crypto.encrypt(PLAIN_DATA.as_bytes()) {
        Ok(_) => (),
        Err(_) => panic!("Failed to encrypt"),
    };
}

#[test]
fn decrypt_data_without_error() {
    let crypto = match Crypto::new(KEY.as_bytes()) {
        Ok(v) => v,
        Err(e) => panic!("Failed to create crypto instance: {}", e),
    };

    let encrypted_data = match crypto.encrypt(PLAIN_DATA.as_bytes()) {
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
    let crypto = match Crypto::new(KEY.as_bytes()) {
        Ok(v) => v,
        Err(e) => panic!("Failed to create crypto instance: {}", e),
    };

    let encrypted_data = match crypto.encrypt(PLAIN_DATA.as_bytes()) {
        Ok(enc) => enc,
        Err(_) => panic!("Failed to encrypt"),
    };

    let new_plain_data = match crypto.decrypt(&encrypted_data) {
        Ok(dec) => dec,
        Err(_) => panic!("Failed to decrypt"),
    };

    assert_eq!(
        match std::str::from_utf8(&new_plain_data) {
            Ok(v) => v,
            Err(_) => "Invalid utf8 sequence",
        },
        PLAIN_DATA
    );
}
