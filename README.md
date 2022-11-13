<p align="center">
  <a href="#readme">
    <img src="./docs/assets/logo.png" height="auto">
  </a>

  <p align="center">
    <a href="https://github.com/valflrt/fencryption-rust/actions/workflows/tests.yml"><img alt="test status" src="https://img.shields.io/github/workflow/status/valflrt/fencryption-rust/tests" /></a>
    <a href="https://docs.rs/fencryption/latest/fencryption_lib/"><img alt="license" src="https://img.shields.io/docsrs/fencryption" /></a>
    <a href="https://crates.io/crates/fencryption"><img alt="license" src="https://img.shields.io/crates/v/fencryption?color=informational" /></a>
    <a href="./LICENSE"><img alt="license" src="https://img.shields.io/github/license/valflrt/fencryption-rust" /></a>
  </p>

  <p align="center">
    <a href="https://github.com/valflrt/fencryption-rust/issues/new"><b>Report Bug</b></a>
    <br />
    <a href="https://github.com/valflrt/fencryption-rust/blob/master/CHANGELOG.md"><b>Changelog</b></a>
    <!-- <br />
    <a href="https://github.com/valflrt/fencryption-rust/releases"><b>Download</b></a> -->
  </p>
</p>

# Fencryption (Rust)

> This is the new version of [Fencryption (typescript)](https://github.com/valflrt/fencryption) but in Rust (Why did I switch language along the way ? Because typescript was not the most suitable language for this kind of project and also because I wanted to try Rust and low-level programming).

Fencryption is program to encrypt and decrypt files and full directories. Note that this project is at an early stage of development.

**THERE IS ABSOLUTELY NO WARRANTY THAT THIS PROGRAM DOES NOT CONTAIN VULNERABILITIES. USE IT AT YOUR OWN RISK.** _This is not supposed to be used in "real conditions" anyway_

## Usage

```
fencryption --help
```

```
A program to encrypt/decrypt files and full directories

Usage: fencryption [OPTIONS] <COMMAND>

Commands:
  encrypt  Encrypt specified file/directory using the passed key
  decrypt  Encrypt specified file/directory using the passed key
  pack     Pack a directory
  unpack   Open a pack
  help     Print this message or the help of the given subcommand(s)

Options:
  -D, --debug    Enable debug log
  -h, --help     Print help information
  -V, --version  Print version information
```

## Limitations

- Pack files can get pretty huge therefore, it is possible that they exceed the maximum file size of some file systems (for example the maximum file size of fat32 is 4GB).

## Todo

- [x] Provide better help about commands
- [x] Add default file encryption features
- [x] Add stream enc-/decryption features
- [x] Setup directory recursive mapping
- [x] Improve log, error handling and cli
- [x] Implement multithreading
- [x] Edit encryption process so the output file is smaller
- [x] Add packing feature: makes a file with all the files packed inside (a pack) that is then encrypted
  - [x] Command `pack` creates a pack from the specified directory
  - [x] Command `unpack` creates a directory where decrypted files (from the encrypted "pack") appear, you can then choose to update the pack with the changes made in the directory or to discard them

<!-- ## Ideas

> things that **_could_** be added in the future

_Nothing for now_ -->
