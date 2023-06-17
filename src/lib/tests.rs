use std::path::PathBuf;

use crate::{crypto::Crypto, tmp::TmpDir, walk_dir::walk_dir};

// Crypto tests

const KEY: &str = "my_super_key";
const PLAIN_DATA: &[u8] = b"hello :)";

#[test]
fn get_original_data_after_encrypting_and_decrypting() {
    let crypto = Crypto::new(KEY);
    let encrypted_data = crypto.encrypt(PLAIN_DATA).unwrap();
    let new_plain_data = crypto.decrypt(&encrypted_data).unwrap();
    assert_eq!(&new_plain_data, PLAIN_DATA);
}

#[test]
#[should_panic]
fn fail_to_decrypt_data_when_key_is_wrong() {
    let crypto1 = Crypto::new(KEY);
    let crypto2 = Crypto::new(&[KEY.as_bytes(), b"nope"].concat());
    let encrypted_data = crypto1.encrypt(PLAIN_DATA).unwrap();
    crypto2.decrypt(&encrypted_data).unwrap();
}

#[test]
fn get_original_data_after_encrypting_and_decrypting_file() {
    let tmp_dir = TmpDir::new().unwrap();
    let crypto = Crypto::new(KEY);

    tmp_dir.write_file("plain", PLAIN_DATA).unwrap();

    crypto
        .encrypt_io(
            &mut tmp_dir.open_readable("plain").unwrap(),
            &mut tmp_dir.create_file("enc").unwrap(),
        )
        .unwrap();
    crypto
        .decrypt_io(
            &mut tmp_dir.open_readable("enc").unwrap(),
            &mut tmp_dir.create_file("dec").unwrap(),
        )
        .unwrap();

    assert_eq!(tmp_dir.read_file("dec").unwrap(), PLAIN_DATA[..]);
}

#[test]
#[should_panic]
fn fail_to_decrypt_file_when_key_is_wrong() {
    let tmp_dir = TmpDir::new().unwrap();

    let crypto1 = Crypto::new(KEY);
    let crypto2 = Crypto::new(&[KEY.as_bytes(), b"nope"].concat());

    tmp_dir.write_file("plain", PLAIN_DATA).unwrap();

    crypto1
        .encrypt_io(
            &mut tmp_dir.open_readable("plain").unwrap(),
            &mut tmp_dir.create_file("enc").unwrap(),
        )
        .unwrap();

    crypto2
        .decrypt_io(
            &mut tmp_dir.open_readable("enc").unwrap(),
            &mut tmp_dir.create_file("dec").unwrap(),
        )
        .unwrap();
}

// WalkDir tests

#[test]
fn walk_directory_and_encounter_every_file_in_it() {
    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.write_file("a", &[]).unwrap();
    tmp_dir.write_file("b", &[]).unwrap();
    tmp_dir.create_dir("c").unwrap();
    tmp_dir.write_file("c/a", &[]).unwrap();
    tmp_dir.write_file("c/b", &[]).unwrap();
    tmp_dir.write_file("d", &[]).unwrap();

    let entries: Vec<PathBuf> = walk_dir(tmp_dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    assert!(entries
        .iter()
        .all(|e| vec!["a", "b", "c", "c/a", "c/b", "d"]
            .iter()
            .map(|p| tmp_dir.path().join(p))
            .find(|p| e == p)
            .is_some()));
}

// TODO Add tmp tests

// TODO Add tests for encrypt/decrypt text
