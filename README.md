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

# Fencryption (rust)

A remake of [Fencryption](https://github.com/valflrt/fencryption) but in rust (because typescript sucks a lil' bit for this kind of project...).

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
- [x] Add io stream enc-/decryption features
- [x] Setup directory recursive mapping
- [x] Improve log and error handling
- [x] Improve cli
- [x] Implement multithreading
- [x] Edit encryption process so the output file is smaller
- [ ] Encrypting makes a file with all the encrypted files packed inside (a "pack")
  - [ ] Command `create` creates a "pack" from the specified directory
  - [ ] Command `open` creates a directory where decrypted files (from the encrypted "pack") appear
  - [ ] Command `close` updates the "pack" with the changes made in the directory

## Ideas

> things that **_could_** be added in the future

_Nothing for now_
