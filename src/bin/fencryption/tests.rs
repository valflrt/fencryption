use fencryption_lib::tmp::TmpDir;

use crate::commands::{decrypt_file, encrypt_file, CommandOutput};

const KEY: &str = "my_super_key";
const PLAIN_DATA: &[u8] = b"hello :)";

#[test]
fn encrypt_and_decrypt_one_file() {
    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.write_file("plain", PLAIN_DATA).unwrap();

    let CommandOutput::EncryptFile { success, .. } = encrypt_file(
        KEY.to_string(),
        &vec![tmp_dir.path().join("plain")],
        &Some(tmp_dir.path().join("enc")),
        false,
        false,
    )
    .unwrap() else { unreachable!() };

    assert_eq!(success, 1);
    assert!(tmp_dir.exists("enc"));

    let CommandOutput::DecryptFile { success, .. } = decrypt_file(
        KEY.to_string(),
        &vec![tmp_dir.path().join("enc")],
        &Some(tmp_dir.path().join("dec")),
        false,
        false,
    )
    .unwrap() else { unreachable!() };

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

    let CommandOutput::EncryptFile { success, .. } = encrypt_file(
        KEY.to_string(),
        &vec![tmp_dir.path().join("plain")],
        &Some(tmp_dir.path().join("enc")),
        false,
        false,
    )
    .unwrap() else { unreachable!() };

    assert_eq!(success, 5);

    assert!(tmp_dir.exists("enc"));
    assert_eq!(tmp_dir.read_dir("enc").unwrap().count(), 5);

    let CommandOutput::DecryptFile { success, .. } = decrypt_file(
        KEY.to_string(),
        &vec![tmp_dir.path().join("enc")],
        &Some(tmp_dir.path().join("dec")),
        false,
        false,
    )
    .unwrap() else { unreachable!() };

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
