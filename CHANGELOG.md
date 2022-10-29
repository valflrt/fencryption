# Changelog

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
