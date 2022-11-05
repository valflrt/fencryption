use crate::crypto::Crypto;
use crate::test::util::TmpDir;

// Crypto tests

const KEY: &[u8] = b"my_super_key";
const PLAIN_DATA: &[u8] = b"hello :)";

#[test]
fn get_original_data_after_encrypting_and_decrypting() {
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
fn get_original_data_after_encrypting_and_decrypting_file() {
    let crypto = Crypto::new(KEY).unwrap();

    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.write_file("plain", PLAIN_DATA).unwrap();
    crypto
        .encrypt_stream(
            &mut tmp_dir.open_file("plain").unwrap(),
            &mut tmp_dir.create_file("enc").unwrap(),
        )
        .unwrap();

    crypto
        .decrypt_stream(
            &mut tmp_dir.open_file("enc").unwrap(),
            &mut tmp_dir.create_file("dec").unwrap(),
        )
        .unwrap();

    assert_eq!(tmp_dir.read_file("dec").unwrap(), PLAIN_DATA[..]);
}

#[test]
#[should_panic]
fn fail_to_decrypt_file_when_key_is_wrong() {
    let crypto1 = Crypto::new(KEY).unwrap();
    let crypto2 = Crypto::new(&[KEY, b"nope"].concat()).unwrap();

    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.write_file("plain", PLAIN_DATA).unwrap();
    crypto1
        .encrypt_stream(
            &mut tmp_dir.open_file("plain").unwrap(),
            &mut tmp_dir.create_file("enc").unwrap(),
        )
        .unwrap();

    crypto2
        .decrypt_stream(
            &mut tmp_dir.open_file("enc").unwrap(),
            &mut tmp_dir.create_file("dec").unwrap(),
        )
        .unwrap();
}

// WalkDir tests
