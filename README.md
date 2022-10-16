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
    <!-- <br />
    <a href="https://github.com/valflrt/fencryption-rust/blob/master/CHANGELOG.md"><b>Changelog</b></a>
    <br />
    <a href="https://github.com/valflrt/fencryption-rust/releases/latest"><b>Download</b></a> -->
  </p>
</p>

# Fencryption (rust)

A remake of [Fencryption](https://github.com/valflrt/fencryption) but in rust (because typescript sucks a lil' bit for this kind of project...).

**THERE IS ABSOLUTELY NO WARRANTY THAT THIS PROGRAM DOES NOT CONTAIN VULNERABILITIES. USE IT AT YOUR OWN RISK.**

## Todo

- [x] Provide better help about commands
- [x] Add default file encryption features
- [x] Add io stream enc-/decryption features
- [x] Setup directory recursive mapping
- [ ] Improve log and error handling
- [ ] Improve cli

### Possible todo _(things that could be added in the future)_

> Those are just some ideas, they are not concrete yet

- [ ] Encrypting would make a .bin file with all the encrypted files packed inside
  - [ ] Command "open" would create a directory where decrypted files appear
  - [ ] Command "close" would update the .bin file with the changes made in the directory
