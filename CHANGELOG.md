# Changelog

## v0.1.7

- Add packing features
  - Command `pack` and `unpack`
- Commands `encrypt` now gathers all encrypted files in the same directory
  - File names of encrypted files are now uuids
- Key is no more asked as an argument (so it is now saved in history)
- Better log

## `v0.1.6`

- Add multithreading for directory encryption (speed goes brrr)
- Switch from chacha20poly1305 to aes_gcm
- Add overwrite option to be able to overwrite the output file/directory when enc/decrypting
- Greatly improve logging
- Filenames are still not encrypted
- Encrypted directory still keeps original directory structure
- Misc fixes

## `v0.1.4`

- Base features (encrypting/decrypting) and commands (encrypt/decrypt)
- No pretty error display (system error message)
- Filenames are not encrypted
- Encrypted directory keeps original directory structure
