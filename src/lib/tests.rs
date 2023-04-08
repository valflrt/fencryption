use std::path::PathBuf;

use crate::{
    commands::{decrypt_file, encrypt_file},
    crypto::Crypto,
    tmp::TmpDir,
    walk_dir::walk_dir,
};

// Crypto tests

const KEY: &str = "my_super_key";
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
    let crypto2 = Crypto::new(&[KEY.as_bytes(), b"nope"].concat()).unwrap();
    let encrypted_data = crypto1.encrypt(PLAIN_DATA).unwrap();
    crypto2.decrypt(&encrypted_data).unwrap();
}

#[test]
fn get_original_data_after_encrypting_and_decrypting_file() {
    let tmp_dir = TmpDir::new().unwrap();
    let crypto = Crypto::new(KEY).unwrap();

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

    let crypto1 = Crypto::new(KEY).unwrap();
    let crypto2 = Crypto::new(&[KEY.as_bytes(), b"nope"].concat()).unwrap();

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

// Tmp tests

// TODO Add tmp tests

// Commands tests

#[test]
fn encrypt_and_decrypt_one_file() {
    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.write_file("plain", b"Hello :)").unwrap();

    let (success, _, _, _) = encrypt_file::execute(
        &KEY.to_string(),
        &vec![tmp_dir.path().join("plain")],
        &Some(tmp_dir.path().join("enc")),
        &false,
        &false,
    )
    .unwrap();

    assert_eq!(success, 1);
    assert!(tmp_dir.exists("enc"));

    let (success, _, _, _) = decrypt_file::execute(
        &KEY.to_string(),
        &vec![tmp_dir.path().join("enc")],
        &Some(tmp_dir.path().join("dec")),
        &false,
        &false,
    )
    .unwrap();

    assert_eq!(success, 1);
    assert!(tmp_dir.exists("dec"));
    assert_eq!(
        tmp_dir.read_file("plain").unwrap(),
        tmp_dir.read_file("dec").unwrap()
    );
}

#[test]
fn encrypt_and_decrypt_several_files() {
    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.create_dir_all("plain/c").unwrap();

    tmp_dir.write_file("plain/a", b"a").unwrap();
    tmp_dir.write_file("plain/b", b"b").unwrap();
    tmp_dir.write_file("plain/c/a", b"c/a").unwrap();
    tmp_dir.write_file("plain/c/b", b"c/b").unwrap();
    tmp_dir.write_file("plain/d", b"d").unwrap();

    let (success, _, _, _) = encrypt_file::execute(
        &KEY.to_string(),
        &vec![tmp_dir.path().join("plain")],
        &Some(tmp_dir.path().join("enc")),
        &false,
        &false,
    )
    .unwrap();

    assert_eq!(success, 5);

    assert!(tmp_dir.exists("enc"));
    assert_eq!(tmp_dir.read_dir("enc").unwrap().count(), 5);

    let (success, _, _, _) = decrypt_file::execute(
        &KEY.to_string(),
        &vec![tmp_dir.path().join("enc")],
        &Some(tmp_dir.path().join("dec")),
        &false,
        &false,
    )
    .unwrap();

    assert_eq!(success, 5);

    assert!(tmp_dir.exists("dec"));
    assert!(tmp_dir.exists("dec/c"));

    assert!(tmp_dir.exists("dec/a"));
    assert!(tmp_dir.exists("dec/b"));
    assert!(tmp_dir.exists("dec/c/a"));
    assert!(tmp_dir.exists("dec/c/b"));
    assert!(tmp_dir.exists("dec/d"));

    assert_eq!(
        tmp_dir.read_file("plain/a").unwrap(),
        tmp_dir.read_file("dec/a").unwrap()
    );
    assert_eq!(
        tmp_dir.read_file("plain/b").unwrap(),
        tmp_dir.read_file("dec/b").unwrap()
    );
    assert_eq!(
        tmp_dir.read_file("plain/c/a").unwrap(),
        tmp_dir.read_file("dec/c/a").unwrap()
    );
    assert_eq!(
        tmp_dir.read_file("plain/c/b").unwrap(),
        tmp_dir.read_file("dec/c/b").unwrap()
    );
    assert_eq!(
        tmp_dir.read_file("plain/d").unwrap(),
        tmp_dir.read_file("dec/d").unwrap()
    );
}

// TODO Add tests for encrypt/decrypt text
