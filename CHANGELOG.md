# Changelog

> v1.0 coming soon ?

## `v0.1.13`

- Fix cli text
- Improve docs

## `v0.1.12`

- Move commands to lib
- Better cli code quality

## `v0.1.11`

- Fix potential security vulnerability by adding one nonce per chunk while keeping encrypted files size pretty low by making chunks longer (so that the chunk/nonce length ratio is low)
- Remove packing feature
- Greatly improve fencryption_lib documention

## `v0.1.10`

- Add packing features
  - Command `pack` and `unpack`
- Commands `encrypt` now gathers all encrypted files in the same directory
  - File names of encrypted files are now uuids
- Key is no more asked as an argument (so it is now saved in history)
- Better log

## `v0.1.6` (beta)

- Add multithreading for directory encryption (speed goes brrr)
- Switch from chacha20poly1305 to aes_gcm
- Add overwrite option to be able to overwrite the output file/directory when enc/decrypting
- Greatly improve logging
- Filenames are still not encrypted
- Encrypted directory still keeps original directory structure
- Misc fixes

## `v0.1.4` (beta)

- Base features (encrypting/decrypting) and commands (encrypt/decrypt)
- No pretty error display (system error message)
- Filenames are not encrypted
- Encrypted directory keeps original directory structure
