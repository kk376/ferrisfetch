# Void Linux Packaging for KKFetch

This directory contains the Void Linux `xbps-src` package template for `kkfetch`.

## Template Location

To build and test within a local `void-packages` tree:

```bash
git clone --depth=1 https://github.com/void-linux/void-packages.git
cd void-packages
./xbps-src binary-bootstrap

# Copy the template into srcpkgs
mkdir -p srcpkgs/kkfetch
cp /path/to/kkfetch/packaging/void/template srcpkgs/kkfetch/

# Build and lint
./xbps-src pkg kkfetch
xlint srcpkgs/kkfetch/template
```

## Submitting to `void-linux/void-packages`

1. Fork and clone `void-linux/void-packages`.
2. Create a branch: `git checkout -b kkfetch`.
3. Add `srcpkgs/kkfetch/template`.
4. Test the build: `./xbps-src pkg kkfetch`.
5. Run linter: `xlint srcpkgs/kkfetch/template`.
6. Commit with the standard Void message format:
   ```bash
   git commit -m "New package: kkfetch-0.11.7"
   ```
7. Push and open a PR against `void-linux/void-packages:master`.
