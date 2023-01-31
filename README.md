<p align="center">
  <a href="#readme">
    <img src="./docs/assets/logo.png" height="auto">
  </a>

  <p align="center">
    <a href="https://github.com/valflrt/fencryption-rust/actions/workflows/tests.yml"><img alt="test status" src="https://img.shields.io/github/actions/workflow/status/valflrt/fencryption-rust/tests.yml" /></a>
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

- [Fencryption (binary)](#fencryption-binary)
  - [Preview](#preview)
  - [Limitations/Issues](#limitationsissues)
  - [Roadmap](#roadmap)
  - [Tests commands should pass](#tests-commands-should-pass)
  - [Ideas](#ideas)
- [fencryption-lib](#fencryption-lib)

# Fencryption (binary)

> This is the new version of [Fencryption (typescript)](https://github.com/valflrt/fencryption) but in Rust (because typescript was not the most suitable language for this kind of project and also because I wanted to try Rust and low-level programming).

Fencryption is program to encrypt and decrypt files and full directories. Note that this project is at an early stage of development.

**THERE IS ABSOLUTELY NO WARRANTY THAT THIS PROGRAM DOES NOT CONTAIN VULNERABILITIES. USE IT AT YOUR OWN RISK.**

This program is not supposed to be used in "real conditions" because it is a mere personal project. Although I think above v1.0 the commands encrypt and decrypt should be pretty safe.

## Preview

```
fencryption --help
```

```
A program to encrypt/decrypt text, files and directories

Usage: fencryption [OPTIONS] <COMMAND>

Commands:
  encrypt  Encrypt text or files and directories
  decrypt  Decrypt text or files and directories
  help     Print this message or the help of the given subcommand(s)

Options:
  -D, --debug    Enable debug log
  -h, --help     Print help
  -V, --version  Print version
```

## Limitations/Issues

- Pack files can get pretty huge therefore it is possible that they exceed the maximum file size of some file systems (for example the maximum file size of fat32 is 4GB).
- When encrypting files, make sure to encrypt and decrypt them with the same version of fencryption (it is very likely that different versions will not work the same way)

## Roadmap

- [x] Provide better help about commands
- [x] Add default file encryption features
- [x] Add stream enc-/decryption features
- [x] Setup directory recursive mapping
- [x] Improve log, error handling and cli
- [x] Implement multithreading
- [x] Edit encryption process so the output file is smaller
- [ ] Add commands to enc/decrypt text/base64/hex
- [ ] Add packing related commands
- [ ] Come up with a stable version (v1.0)

## Tests commands should pass

- encrypt and decrypt
  - [x] encrypt/decrypt
  - [x] can encrypt/decrypt several items at once
  - [x] can set to a custom output path
    - [x] only when there is one input path
  - [x] overwrite when asked
  - [x] delete original when asked
  - [x] print debug log when asked

## Ideas

> things that **_could_** be added in the future

- Packing: makes a file with all the files packed inside (a pack) that is then encrypted
  - Command `pack create` creates a pack from the contents of specified directory
  - Command `pack update` creates a directory where decrypted files (from the encrypted "pack") appear, you can then choose to update the pack with the changes made in the directory or to discard them
  - Command `pack extract` extracts pack to the specified directory

# fencryption-lib

The lib used in the fencryption binary. You can [take a look](https://docs.rs/fencryption/latest/fencryption_lib/), it has some interesting things...
