# rust-file

`rust-file` is a small Rust command-line archive tool. It packs the files in a
directory into a custom archive format, compresses the archive with zstd, and
encrypts it with XChaCha20-Poly1305 using a key derived from your password with
Argon2.

It also includes simple TCP send/receive commands and UDP discovery helpers for
finding a receiver on the local network.

## Status

This is an experimental project. Be careful with real data.

Important: `--enc --dir <directory_path>` deletes the source directory after it
writes the encrypted archive.

## Build

```sh
cargo build
```

For an optimized binary:

```sh
cargo build --release
```

Run tests and lints:

```sh
cargo test
cargo clippy --all-targets --all-features
```

## Usage

Show help by running the binary without a valid command:

```sh
cargo run --
```

### Encrypt a directory

```sh
cargo run -- --enc --dir <directory_path>
```

The program asks for:

- a password
- a zstd compression level from `-7` to `22`

It writes an archive named like:

```text
mole_<random>_<pid>_<timestamp>.bin
```

After writing the archive, the source directory is removed.

### Decrypt an archive

```sh
cargo run -- --dec <archive_path> [output_directory]
```

If `output_directory` is omitted, files are restored into the current directory.

### Send an encrypted archive over TCP

The sender can send the encrypted archive after creating it:

```sh
cargo run -- --enc --dir <directory_path> --send <host:port>
```

Example:

```sh
cargo run -- --enc --dir ./secret --send 192.168.1.20:9000
```

### Receive an archive over TCP

On the receiving machine:

```sh
cargo run -- --receive <host:port>
```

Example:

```sh
cargo run -- --receive 0.0.0.0:9000
```

The received bytes are written to a generated `mole_*.bin` archive file.

The old misspelled command `--recieve` is also accepted.

### Receiver discovery

Discovery uses UDP port `9001`.

On the receiver, advertise a file name and the TCP address where it can receive:

```sh
cargo run -- --discover-serve <file_name> <host:port>
```

Example:

```sh
cargo run -- --discover-serve backup.bin 192.168.1.20:9000
```

On another machine, search for that receiver:

```sh
cargo run -- --find-receiver <file_name>
```

Example:

```sh
cargo run -- --find-receiver backup.bin
```

If a matching receiver answers within 3 seconds, the command prints the receiver
address.

## Archive Format

The encrypted file contains:

1. 16-byte random salt
2. 24-byte random nonce
3. encrypted zstd-compressed archive bytes

Inside the compressed archive, metadata is written in little-endian fields,
followed by file names, relative paths, and file contents.


## License

Apache License 2.0. See [LICENSE](LICENSE).
