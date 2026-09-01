# KISS Linux Packaging for KKFetch

This directory contains the KISS Linux package recipe files for `kkfetch`.

## Maintainer

- Kushagra Kumar (kk376)

## Package Structure

- `build`: Posix shell script executing `cargo build --release --locked` and installing binaries, shell completions, documentation, and license.
- `version`: Package version and release (`0.11.7 1`).
- `sources`: Source tarball location (`https://github.com/kk376/kkfetch/archive/refs/tags/v0.11.7.tar.gz`).
- `checksums`: SHA256 checksum for source tarball validation.
- `depends`: Build/runtime dependencies (`rust make`).

## Building & Testing Locally

1. Add this directory or your custom repository to `$KISS_PATH`:
   ```sh
   export KISS_PATH="/path/to/kkfetch/packaging/kiss:$KISS_PATH"
   ```

2. Build and install:
   ```sh
   kiss build kkfetch
   kiss install kkfetch
   ```

3. Generate or verify checksums:
   ```sh
   kiss checksum kkfetch
   ```

## Upstream Community Repository

This package is maintained in the KISS Community Repository:
- [kiss-community/community](https://github.com/kiss-community/community)
