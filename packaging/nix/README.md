# Nix Packaging for KKFetch

This directory contains Nix packaging manifests for KKFetch following standard Nixpkgs conventions.

## Manifests

- [`package.nix`](package.nix): Standard Nixpkgs derivation using `rustPlatform.buildRustPackage` with `installShellFiles` for completions.
- [`default.nix`](default.nix): Entrypoint for local `nix-build` targeting the workspace source.
- [`flake.nix`](flake.nix): Modern Nix Flake definition providing packages, apps, devShells, and overlays across Linux architectures (`x86_64`, `aarch64`, `i686`, `riscv64`).

## Usage

### Build with Flakes

```bash
# Build package
nix build --impure ./packaging/nix

# Run directly
nix run --impure ./packaging/nix -- --help
```

### Build with Legacy Nix (`nix-build`)

```bash
nix-build packaging/nix
./result/bin/kkfetch
```

### Development Shell

```bash
nix develop --impure ./packaging/nix
```

### Nixpkgs Inclusion

To submit KKFetch to upstream Nixpkgs (`nixpkgs/pkgs/by-name/kk/kkfetch/package.nix`):

1. Set `hash` in `src = fetchFromGitHub { ... }` to the release source tarball SRI hash.
2. Replace `cargoLock.lockFile` with `cargoHash = "sha256-...";` (generated using `nix-prefetch-github` or running the build once to obtain the fixed-output derivation hash).
3. Open a PR against `NixOS/nixpkgs` targeting `master`.
