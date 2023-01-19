use fencryption_lib::tmp::TmpDir;

use crate::executions;

const KEY: &str = "test";

#[test]
fn get_original_after_enc_and_dec() {
    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.create_dir_all("1/1.2").unwrap();

    tmp_dir.write_file("1/1.1", b"1.1").unwrap();
    tmp_dir.write_file("1/1.3", b"1.3").unwrap();

    tmp_dir.write_file("1/1.2/1.2.1", b"1.2.1").unwrap();
    tmp_dir.write_file("1/1.2/1.2.2", b"1.2.2").unwrap();

    tmp_dir.write_file("2", b"2").unwrap();

    executions::encrypt(
        vec![tmp_dir.path().join("1"), tmp_dir.path().join("2")],
        None,
        KEY.to_string(),
        false,
        false,
    )
    .unwrap();

    assert!(tmp_dir.exists("1.enc"));
    assert!(tmp_dir.exists("2.enc"));

    assert_eq!(tmp_dir.read_dir().unwrap().fold(0, |a, _| a + 1), 4);

    executions::decrypt(
        vec![tmp_dir.path().join("1.enc"), tmp_dir.path().join("2.enc")],
        None,
        KEY.to_string(),
        false,
        false,
    )
    .unwrap();

    assert!(tmp_dir.exists("1.dec"));
    assert!(tmp_dir.exists("2.dec"));

    assert_eq!(tmp_dir.read_file("1.dec/1.1").unwrap(), b"1.1");
    assert_eq!(tmp_dir.read_file("1.dec/1.3").unwrap(), b"1.3");

    assert_eq!(tmp_dir.read_file("1.dec/1.2/1.2.1").unwrap(), b"1.2.1");
    assert_eq!(tmp_dir.read_file("1.dec/1.2/1.2.2").unwrap(), b"1.2.2");

    assert_eq!(tmp_dir.read_file("2.dec").unwrap(), b"2");
}

#[test]
fn overwrite_output_dirs() {
    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.create_dir_all("1/1.2").unwrap();

    tmp_dir.write_file("1/1.1", b"1.1").unwrap();
    tmp_dir.write_file("1/1.3", b"1.3").unwrap();

    tmp_dir.write_file("1/1.2/1.2.1", b"1.2.1").unwrap();
    tmp_dir.write_file("1/1.2/1.2.2", b"1.2.2").unwrap();

    tmp_dir.create_dir("1.enc").unwrap();
    tmp_dir.write_file("1.enc/hello", &[]).unwrap();

    executions::encrypt(
        vec![tmp_dir.path().join("1")],
        None,
        KEY.to_string(),
        true,
        false,
    )
    .unwrap();

    assert!(!tmp_dir.exists("1.enc/hello"));

    tmp_dir.create_dir("1.dec").unwrap();
    tmp_dir.write_file("1.dec/hello2", &[]).unwrap();

    executions::decrypt(
        vec![tmp_dir.path().join("1.enc")],
        None,
        KEY.to_string(),
        true,
        false,
    )
    .unwrap();

    assert!(!tmp_dir.exists("1.dec/hello2"));
}

#[test]
fn delete_original_directory_when_enc_dec_if_asked() {
    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.create_dir_all("1").unwrap();
    tmp_dir.write_file("1/1.1", b"1.1").unwrap();

    executions::encrypt(
        vec![tmp_dir.path().to_path_buf()],
        None,
        KEY.to_string(),
        false,
        true,
    )
    .unwrap();

    assert!(!tmp_dir.exists("1"));

    executions::decrypt(
        vec![tmp_dir.path().join("1.enc")],
        None,
        KEY.to_string(),
        false,
        true,
    )
    .unwrap();

    assert!(!tmp_dir.exists("1.enc"));
}

#[test]
fn get_original_after_pack_and_unpack() {
    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.create_dir_all("1/1.2").unwrap();

    tmp_dir.write_file("1/1.1", b"1.1").unwrap();
    tmp_dir.write_file("1/1.3", b"1.3").unwrap();

    tmp_dir.write_file("1/1.2/1.2.1", b"1.2.1").unwrap();
    tmp_dir.write_file("1/1.2/1.2.2", b"1.2.2").unwrap();

    tmp_dir.write_file("2", b"2").unwrap();

    executions::pack(tmp_dir.path().join("1"), KEY.to_string(), false, false).unwrap();

    assert!(tmp_dir.exists("1"));
    assert!(tmp_dir.exists("1.pack"));

    executions::unpack(
        tmp_dir.path().join("1.pack"),
        tmp_dir.path().join("1.dec"),
        KEY.to_string(),
    )
    .unwrap();

    assert!(tmp_dir.exists("1.dec"));

    assert_eq!(tmp_dir.read_file("1.dec/1.1").unwrap(), b"1.1");
    assert_eq!(tmp_dir.read_file("1.dec/1.3").unwrap(), b"1.3");

    assert_eq!(tmp_dir.read_file("1.dec/1.2/1.2.1").unwrap(), b"1.2.1");
    assert_eq!(tmp_dir.read_file("1.dec/1.2/1.2.2").unwrap(), b"1.2.2");
}

#[test]
fn delete_original_directory_when_creating_pack() {
    let tmp_dir = TmpDir::new().unwrap();

    tmp_dir.create_dir("1").unwrap();
    tmp_dir.write_file("1/1.1", b"1.1").unwrap();

    executions::pack(tmp_dir.path().join("1"), KEY.to_string(), true, true).unwrap();

    assert!(!tmp_dir.exists("1"));
}
