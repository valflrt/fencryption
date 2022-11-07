<p align="center">
  <a href="#readme">
    <img src="./docs/assets/logo.png" height="auto">
  </a>

  <p align="center">
    <a href="https://github.com/valflrt/fencryption-rust/actions/workflows/tests.yml"><img alt="test status" src="https://img.shields.io/github/workflow/status/valflrt/fencryption-rust/tests" /></a>
    <a href="./LICENSE"><img alt="license" src="https://img.shields.io/github/license/valflrt/fencryption-rust" /></a>
  </p>

  <p align="center">
    <a href="https://github.com/valflrt/fencryption-rust/issues/new"><b>Report Bug</b></a>
    <br />
    <a href="https://github.com/valflrt/fencryption-rust/blob/master/CHANGELOG.md"><b>Changelog</b></a>
    <!-- <br />
    <a href="https://github.com/valflrt/fencryption-rust/releases/latest"><b>Download</b></a> -->
  </p>
</p>

# Fencryption (Rust)

> This is the new version of [Fencryption](https://github.com/valflrt/fencryption) but in Rust (Why did I switch language along the way ? because typescript was not the most suitable language for this kind of project and also because I wanted to try Rust and low-level programming)

Fencryption is program to encrypt and decrypt files and full directories. Note that this project is at an early stage of development.

**THERE IS ABSOLUTELY NO WARRANTY THAT THIS PROGRAM DOES NOT CONTAIN VULNERABILITIES. USE IT AT YOUR OWN RISK.**

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
  help     Print this message or the help of the given subcommand(s)

Options:
  -D, --debug    Enable debug log
  -h, --help     Print help information
  -V, --version  Print version information
```

## Todo

- [x] Provide better help about commands
- [x] Add default file encryption features
- [x] Add stream enc-/decryption features
- [x] Setup directory recursive mapping
- [x] Improve log, error handling and cli
- [x] Implement multithreading
- [x] Edit encryption process so the output file is smaller
- [x] Add "packing" feature: encrypting makes a file with all the encrypted files packed inside (a "pack")
  - [x] Command `pack` creates a "pack" with the specified directory
  - [x] Command `unpack` creates a directory where decrypted files (from the encrypted "pack") appear, updates the "pack" with the changes made in the directory (or discards them)

## Ideas

> things that **_could_** be added in the future

_Nothing for now_
