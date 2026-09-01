# Packaging KKFetch

This directory contains package definitions, build scripts, and metadata for Linux, macOS, BSD, and Windows package managers.

## Package Matrix

| Distribution / Repository | Recipe Location | Primary Tool | Target Output |
| :--- | :--- | :--- | :--- |
| **Arch Linux / AUR** | [`arch/PKGBUILD`](arch/PKGBUILD) | `makepkg` | `.pkg.tar.zst` |
| **Debian / Ubuntu** | [`debian/`](debian/) | `dpkg-buildpackage` / `cargo-deb` | `.deb` |
| **Fedora / RHEL / Copr** | [`rpm/kkfetch.spec`](rpm/kkfetch.spec) | `rpmbuild` / `cargo-generate-rpm` | `.rpm` |
| **Android / Termux** | [`termux/build.sh`](termux/build.sh) | `termux-packages` | Termux `.deb` |
| **Nix / NixOS** | [`nix/default.nix`](nix/default.nix), [`nix/flake.nix`](nix/flake.nix) | `nix build` | Nix store path |
| **Void Linux** | [`void/template`](void/template) | `xbps-src` | `.xbps` |
| **Alpine Linux** | [`alpine/APKBUILD`](alpine/APKBUILD) | `abuild` | `.apk` |
| **Gentoo Linux** | [`gentoo/kkfetch-0.10.0.ebuild`](gentoo/) | `ebuild` / `emerge` | Portage ebuild |
| **Homebrew** | [`homebrew/kkfetch.rb`](homebrew/kkfetch.rb) | `brew` | Formula / Bottled bottle |
| **KISS Linux** | [`kiss/`](kiss/) | `kiss` | KISS package |
| **Windows Package Manager (Winget)** | [`winget/kkfetch.yaml`](winget/kkfetch.yaml) | `winget` | Winget manifest |

---

## Distribution Recipes and Build Workflows

### 1. Arch Linux (AUR)

- **Source files**: [`packaging/arch/PKGBUILD`](arch/PKGBUILD), [`packaging/arch/.SRCINFO`](arch/.SRCINFO)
- **Install paths**:
  - Binary: `/usr/bin/kkfetch`
  - Shell completions: `/usr/share/bash-completion/completions/kkfetch`, `/usr/share/zsh/site-functions/_kkfetch`, `/usr/share/fish/vendor_completions.d/kkfetch.fish`
  - Docs & license: `/usr/share/doc/kkfetch/README.md`, `/usr/share/licenses/kkfetch/LICENSE`

#### Local Build
```bash
cd packaging/arch
makepkg -si
```

To verify in a clean chroot:
```bash
extra-x86_64-build
```

#### AUR Submission
1. Clone the AUR repository:
   ```bash
   git clone ssh://aur@aur.archlinux.org/kkfetch.git
   ```
2. Copy `PKGBUILD` into the repository.
3. Update checksums and regenerate `.SRCINFO`:
   ```bash
   updpkgsums
   makepkg --printsrcinfo > .SRCINFO
   ```
4. Commit and push:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Update to version 0.10.0"
   git push origin master
   ```

---

### 2. Debian & Ubuntu

- **Source files**: [`packaging/debian/`](debian/) (`control`, `rules`, `changelog`, `compat`, `copyright`, `source/format`)
- **Install paths**:
  - Binary: `/usr/bin/kkfetch`
  - Shell completions: `/usr/share/bash-completion/completions/kkfetch`, `/usr/share/zsh/vendor-completions/_kkfetch`, `/usr/share/fish/vendor_completions.d/kkfetch.fish`
  - Docs & copyright: `/usr/share/doc/kkfetch/README.md`, `/usr/share/doc/kkfetch/copyright`

#### Local Build with Debian Tooling
```bash
# From the repository root
dpkg-buildpackage -us -uc -b
```

#### Local Build with cargo-deb
```bash
cargo install cargo-deb
cargo deb
```
Output: `target/debian/kkfetch_0.10.0-1_amd64.deb`

#### Debian Submission
1. Create a signed source package:
   ```bash
   dpkg-buildpackage -S -sa -k<GPG-KEY-ID>
   ```
2. Run lintian to verify compliance:
   ```bash
   lintian --pedantic -I kkfetch_0.10.0-1.dsc
   ```
3. Upload to Debian Mentors:
   ```bash
   dput mentors kkfetch_0.10.0-1_source.changes
   ```
4. File an RFS (Request for Sponsorship) bug against `sponsorship-requests` on Debian BTS.

---

### 3. Fedora, RHEL & Copr (RPM)

- **Source files**: [`packaging/rpm/kkfetch.spec`](rpm/kkfetch.spec)
- **Install paths**:
  - Binary: `%{_bindir}/kkfetch`
  - Shell completions: `%{_datadir}/bash-completion/completions/kkfetch`, `%{_datadir}/zsh/site-functions/_kkfetch`, `%{_datadir}/fish/vendor_completions.d/kkfetch.fish`
  - Docs & license: `%{_docdir}/kkfetch/README.md`, `%{_licensedir}/kkfetch/LICENSE`

#### Local Build with rpmbuild
```bash
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cp packaging/rpm/kkfetch.spec ~/rpmbuild/SPECS/
spectool -g -R ~/rpmbuild/SPECS/kkfetch.spec
rpmbuild -ba ~/rpmbuild/SPECS/kkfetch.spec
```

#### Local Build with cargo-generate-rpm
```bash
cargo install cargo-generate-rpm
cargo build --release
cargo generate-rpm
```
Output: `target/generate-rpm/kkfetch-0.10.0-1.x86_64.rpm`

#### Copr Submission
1. Install Copr CLI:
   ```bash
   sudo dnf install copr-cli
   ```
2. Build in your Copr repository:
   ```bash
   copr-cli build kk376/kkfetch ~/rpmbuild/SRPMS/kkfetch-0.10.0-1.*.src.rpm
   ```
   Or set up a GitHub webhook to build automatically on git release tags.

---

### 4. Android (Termux)

- **Source files**: [`packaging/termux/build.sh`](termux/build.sh)
- **Install paths**:
  - Binary: `$PREFIX/bin/kkfetch`
  - Completions: `$PREFIX/share/bash-completion/completions/kkfetch`, `$PREFIX/share/zsh/site-functions/_kkfetch`, `$PREFIX/share/fish/vendor_completions.d/kkfetch.fish`

#### Local Build in Termux Environment
```bash
# Inside termux-packages repository clone:
./scripts/run-docker.sh ./build-package.sh -a aarch64 kkfetch
```

#### Termux User Repository (TUR) Submission
1. Fork `termux-user-repository/tur` on GitHub.
2. Create `packages/kkfetch/build.sh` using the recipe in [`packaging/termux/build.sh`](termux/build.sh).
3. Test the build locally with `./start-builder.sh ./build-package.sh kkfetch`.
4. Open a pull request against `termux-user-repository/tur`.

---

### 5. Nix / NixOS

- **Source files**: [`packaging/nix/package.nix`](nix/package.nix), [`packaging/nix/default.nix`](nix/default.nix), [`packaging/nix/flake.nix`](nix/flake.nix)

#### Local Build
Using Nix Flakes:
```bash
nix build packaging/nix#kkfetch
```

Using legacy Nix expressions:
```bash
nix-build packaging/nix/default.nix
```

Run directly without installing:
```bash
nix run github:kk376/kkfetch
```

#### Nixpkgs Submission
1. Fork `NixOS/nixpkgs` on GitHub.
2. Place the derivation in `pkgs/by-name/kk/kkfetch/package.nix`.
3. Add to `pkgs/top-level/all-packages.nix` if applicable.
4. Test build:
   ```bash
   nix-build -A kkfetch
   ```
5. Submit a pull request to `NixOS/nixpkgs:master`.

---

### 6. Void Linux

- **Source files**: [`packaging/void/template`](void/template)

#### Local Build with xbps-src
```bash
git clone --depth=1 https://github.com/void-linux/void-packages.git
cd void-packages
./xbps-src binary-bootstrap
mkdir -p srcpkgs/kkfetch
cp /path/to/kkfetch/packaging/void/template srcpkgs/kkfetch/
./xbps-src pkg kkfetch
```

#### Void Packages Submission
1. Fork and clone `void-linux/void-packages`.
2. Place `template` in `srcpkgs/kkfetch/template`.
3. Run linter:
   ```bash
   xlint srcpkgs/kkfetch/template
   ```
4. Build and install into a chroot test:
   ```bash
   ./xbps-src -N pkg kkfetch
   ```
5. Submit a pull request to `void-linux/void-packages`.

---

### 7. Alpine Linux

- **Source files**: [`packaging/alpine/APKBUILD`](alpine/APKBUILD)

#### Local Build with abuild
```bash
cd packaging/alpine
abuild checksum
abuild -r
```

#### Alpine aports Submission
1. Fork `alpinelinux/aports` on GitLab (`gitlab.alpinelinux.org/alpine/aports`).
2. Place `APKBUILD` into `testing/kkfetch/APKBUILD`.
3. Lint and test build:
   ```bash
   apkbuild-lint APKBUILD
   abuild -r
   ```
4. Submit a merge request to Alpine's GitLab repository.

---

### 8. Gentoo Linux

- **Source files**: [`packaging/gentoo/kkfetch-0.10.0.ebuild`](gentoo/)

#### Local Build with ebuild
```bash
ebuild packaging/gentoo/kkfetch-0.10.0.ebuild digest
ebuild packaging/gentoo/kkfetch-0.10.0.ebuild clean compile install
```

#### Gentoo Overlay / Main Repository Submission
1. Test with `pkgcheck`:
   ```bash
   pkgcheck scan kkfetch-0.10.0.ebuild
   ```
2. Place in a custom overlay under `app-misc/kkfetch/` or submit a pull request to `gentoo/gentoo` on GitHub.

---

### 9. Homebrew

- **Source files**: [`packaging/homebrew/kkfetch.rb`](homebrew/kkfetch.rb)

#### Local Build & Test
```bash
brew install --build-from-source packaging/homebrew/kkfetch.rb
brew test kkfetch
brew audit --strict packaging/homebrew/kkfetch.rb
```

#### Tap Setup
1. Create a repository named `homebrew-tap` on GitHub (`github.com/kk376/homebrew-tap`).
2. Add `Formula/kkfetch.rb`.
3. Users can then install directly via:
   ```bash
   brew install kk376/tap/kkfetch
   ```

---

### 10. KISS Linux

- **Source files**: [`packaging/kiss/`](kiss/) (`build`, `version`, `sources`, `checksums`, `depends`)
- **Install paths**:
  - Binary: `/usr/bin/kkfetch`
  - Shell completions: `/usr/share/bash-completion/completions/kkfetch`, `/usr/share/zsh/site-functions/_kkfetch`, `/usr/share/fish/vendor_completions.d/kkfetch.fish`
  - Docs & license: `/usr/share/doc/kkfetch/README.md`, `/usr/share/licenses/kkfetch/LICENSE`

#### Local Build with KISS
```bash
export KISS_PATH="/path/to/kkfetch/packaging/kiss:$KISS_PATH"
kiss build kkfetch
kiss install kkfetch
```

#### KISS Community Repository Submission
1. Fork and clone `kiss-community/community`.
2. Create branch `kkfetch`.
3. Place package files in `community/kkfetch/` (`build`, `version`, `sources`, `checksums`, `depends`).
4. Commit with message `kkfetch: new package at 0.10.0`.
5. Open a pull request against `kiss-community/community:main`.

---

### 11. Windows Package Manager (Winget)

- **Source files**: [`packaging/winget/kkfetch.yaml`](winget/kkfetch.yaml), [`packaging/winget/kkfetch.installer.yaml`](winget/kkfetch.installer.yaml), [`packaging/winget/kkfetch.locale.en-US.yaml`](winget/kkfetch.locale.en-US.yaml)

#### Manifest Validation
```powershell
winget-pkgs/tools/YamlCreate.ps1
winget validate packaging/winget
```

#### Winget Submission
1. Fork `microsoft/winget-pkgs` on GitHub.
2. Place manifests under `manifests/k/kk376/kkfetch/0.10.0/`.
3. Open a pull request to `microsoft/winget-pkgs`.

---

## Release Checklist for Package Maintainers

1. Tag the release: `git tag -a v0.10.0 -m "Release v0.10.0"` and push tags to GitHub.
2. Generate source archive checksum:
   ```bash
   curl -sL https://github.com/kk376/kkfetch/archive/refs/tags/v0.10.0.tar.gz | sha256sum
   ```
3. Update version strings and `sha256` checksums across `PKGBUILD`, `.SRCINFO`, `kkfetch.spec`, `build.sh`, `default.nix`, `template`, `APKBUILD`, `ebuild`, `kkfetch.rb`, and `template.py`.
4. Trigger GitHub release artifacts and update distribution package feeds.
