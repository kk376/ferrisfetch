# Packaging FerrisFetch

This directory contains package definitions, build scripts, and metadata for Linux and BSD package managers.

## Package Matrix

| Distribution / Repository | Recipe Location | Primary Tool | Target Output |
| :--- | :--- | :--- | :--- |
| **Arch Linux / AUR** | [`arch/PKGBUILD`](arch/PKGBUILD) | `makepkg` | `.pkg.tar.zst` |
| **Debian / Ubuntu** | [`debian/`](debian/) | `dpkg-buildpackage` / `cargo-deb` | `.deb` |
| **Fedora / RHEL / Copr** | [`rpm/ferrisfetch.spec`](rpm/ferrisfetch.spec) | `rpmbuild` / `cargo-generate-rpm` | `.rpm` |
| **Android / Termux** | [`termux/build.sh`](termux/build.sh) | `termux-packages` | Termux `.deb` |
| **Nix / NixOS** | [`nix/default.nix`](nix/default.nix), [`nix/flake.nix`](nix/flake.nix) | `nix build` | Nix store path |
| **Void Linux** | [`void/template`](void/template) | `xbps-src` | `.xbps` |
| **Alpine Linux** | [`alpine/APKBUILD`](alpine/APKBUILD) | `abuild` | `.apk` |
| **Gentoo Linux** | [`gentoo/ferrisfetch-0.9.8.ebuild`](gentoo/ferrisfetch-0.9.8.ebuild) | `ebuild` / `emerge` | Portage ebuild |
| **Homebrew** | [`homebrew/ferrisfetch.rb`](homebrew/ferrisfetch.rb) | `brew` | Formula / Bottled bottle |
| **KISS Linux** | [`kiss/`](kiss/) | `kiss` | KISS package |

---

## Distribution Recipes and Build Workflows

### 1. Arch Linux (AUR)

- **Source files**: [`packaging/arch/PKGBUILD`](arch/PKGBUILD), [`packaging/arch/.SRCINFO`](arch/.SRCINFO)
- **Install paths**:
  - Binary: `/usr/bin/ferrisfetch`
  - Shell completions: `/usr/share/bash-completion/completions/ferrisfetch`, `/usr/share/zsh/site-functions/_ferrisfetch`, `/usr/share/fish/vendor_completions.d/ferrisfetch.fish`
  - Docs & license: `/usr/share/doc/ferrisfetch/README.md`, `/usr/share/licenses/ferrisfetch/LICENSE`

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
   git clone ssh://aur@aur.archlinux.org/ferrisfetch.git
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
   git commit -m "Update to version 0.9.8"
   git push origin master
   ```

---

### 2. Debian & Ubuntu

- **Source files**: [`packaging/debian/`](debian/) (`control`, `rules`, `changelog`, `compat`, `copyright`, `source/format`)
- **Install paths**:
  - Binary: `/usr/bin/ferrisfetch`
  - Shell completions: `/usr/share/bash-completion/completions/ferrisfetch`, `/usr/share/zsh/vendor-completions/_ferrisfetch`, `/usr/share/fish/vendor_completions.d/ferrisfetch.fish`
  - Docs & copyright: `/usr/share/doc/ferrisfetch/README.md`, `/usr/share/doc/ferrisfetch/copyright`

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
Output: `target/debian/ferrisfetch_0.9.8-1_amd64.deb`

#### Debian Submission
1. Create a signed source package:
   ```bash
   dpkg-buildpackage -S -sa -k<GPG-KEY-ID>
   ```
2. Run lintian to verify compliance:
   ```bash
   lintian --pedantic -I ferrisfetch_0.9.8-1.dsc
   ```
3. Upload to Debian Mentors:
   ```bash
   dput mentors ferrisfetch_0.9.8-1_source.changes
   ```
4. File an RFS (Request for Sponsorship) bug against `sponsorship-requests` on Debian BTS.

---

### 3. Fedora, RHEL & Copr (RPM)

- **Source files**: [`packaging/rpm/ferrisfetch.spec`](rpm/ferrisfetch.spec)
- **Install paths**:
  - Binary: `%{_bindir}/ferrisfetch`
  - Shell completions: `%{_datadir}/bash-completion/completions/ferrisfetch`, `%{_datadir}/zsh/site-functions/_ferrisfetch`, `%{_datadir}/fish/vendor_completions.d/ferrisfetch.fish`
  - Docs & license: `%{_docdir}/ferrisfetch/README.md`, `%{_licensedir}/ferrisfetch/LICENSE`

#### Local Build with rpmbuild
```bash
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cp packaging/rpm/ferrisfetch.spec ~/rpmbuild/SPECS/
spectool -g -R ~/rpmbuild/SPECS/ferrisfetch.spec
rpmbuild -ba ~/rpmbuild/SPECS/ferrisfetch.spec
```

#### Local Build with cargo-generate-rpm
```bash
cargo install cargo-generate-rpm
cargo build --release
cargo generate-rpm
```
Output: `target/generate-rpm/ferrisfetch-0.9.8-1.x86_64.rpm`

#### Copr Submission
1. Install Copr CLI:
   ```bash
   sudo dnf install copr-cli
   ```
2. Build in your Copr repository:
   ```bash
   copr-cli build kk376/ferrisfetch ~/rpmbuild/SRPMS/ferrisfetch-0.9.8-1.*.src.rpm
   ```
   Or set up a GitHub webhook to build automatically on git release tags.

---

### 4. Android (Termux)

- **Source files**: [`packaging/termux/build.sh`](termux/build.sh)
- **Install paths**:
  - Binary: `$PREFIX/bin/ferrisfetch`
  - Completions: `$PREFIX/share/bash-completion/completions/ferrisfetch`, `$PREFIX/share/zsh/site-functions/_ferrisfetch`, `$PREFIX/share/fish/vendor_completions.d/ferrisfetch.fish`

#### Local Build in Termux Environment
```bash
# Inside termux-packages repository clone:
./scripts/run-docker.sh ./build-package.sh -a aarch64 ferrisfetch
```

#### Termux User Repository (TUR) Submission
1. Fork `termux-user-repository/tur` on GitHub.
2. Create `packages/ferrisfetch/build.sh` using the recipe in [`packaging/termux/build.sh`](termux/build.sh).
3. Test the build locally with `./start-builder.sh ./build-package.sh ferrisfetch`.
4. Open a pull request against `termux-user-repository/tur`.

---

### 5. Nix / NixOS

- **Source files**: [`packaging/nix/package.nix`](nix/package.nix), [`packaging/nix/default.nix`](nix/default.nix), [`packaging/nix/flake.nix`](nix/flake.nix)

#### Local Build
Using Nix Flakes:
```bash
nix build packaging/nix#ferrisfetch
```

Using legacy Nix expressions:
```bash
nix-build packaging/nix/default.nix
```

Run directly without installing:
```bash
nix run github:kk376/ferrisfetch
```

#### Nixpkgs Submission
1. Fork `NixOS/nixpkgs` on GitHub.
2. Place the derivation in `pkgs/by-name/fe/ferrisfetch/package.nix`.
3. Add to `pkgs/top-level/all-packages.nix` if applicable.
4. Test build:
   ```bash
   nix-build -A ferrisfetch
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
mkdir -p srcpkgs/ferrisfetch
cp /path/to/ferrisfetch/packaging/void/template srcpkgs/ferrisfetch/
./xbps-src pkg ferrisfetch
```

#### Void Packages Submission
1. Fork and clone `void-linux/void-packages`.
2. Place `template` in `srcpkgs/ferrisfetch/template`.
3. Run linter:
   ```bash
   xlint srcpkgs/ferrisfetch/template
   ```
4. Build and install into a chroot test:
   ```bash
   ./xbps-src -N pkg ferrisfetch
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
2. Place `APKBUILD` into `testing/ferrisfetch/APKBUILD`.
3. Lint and test build:
   ```bash
   apkbuild-lint APKBUILD
   abuild -r
   ```
4. Submit a merge request to Alpine's GitLab repository.

---

### 8. Gentoo Linux

- **Source files**: [`packaging/gentoo/ferrisfetch-0.9.8.ebuild`](gentoo/ferrisfetch-0.9.8.ebuild)

#### Local Build with ebuild
```bash
ebuild packaging/gentoo/ferrisfetch-0.9.8.ebuild digest
ebuild packaging/gentoo/ferrisfetch-0.9.8.ebuild clean compile install
```

#### Gentoo Overlay / Main Repository Submission
1. Test with `pkgcheck`:
   ```bash
   pkgcheck scan ferrisfetch-0.9.8.ebuild
   ```
2. Place in a custom overlay under `app-misc/ferrisfetch/` or submit a pull request to `gentoo/gentoo` on GitHub.

---

### 9. Homebrew

- **Source files**: [`packaging/homebrew/ferrisfetch.rb`](homebrew/ferrisfetch.rb)

#### Local Build & Test
```bash
brew install --build-from-source packaging/homebrew/ferrisfetch.rb
brew test ferrisfetch
brew audit --strict packaging/homebrew/ferrisfetch.rb
```

#### Tap Setup
1. Create a repository named `homebrew-tap` on GitHub (`github.com/kk376/homebrew-tap`).
2. Add `Formula/ferrisfetch.rb`.
3. Users can then install directly via:
   ```bash
   brew install kk376/tap/ferrisfetch
   ```

---

### 10. KISS Linux

- **Source files**: [`packaging/kiss/`](kiss/) (`build`, `version`, `sources`, `checksums`, `depends`)
- **Install paths**:
  - Binary: `/usr/bin/ferrisfetch`
  - Shell completions: `/usr/share/bash-completion/completions/ferrisfetch`, `/usr/share/zsh/site-functions/_ferrisfetch`, `/usr/share/fish/vendor_completions.d/ferrisfetch.fish`
  - Docs & license: `/usr/share/doc/ferrisfetch/README.md`, `/usr/share/licenses/ferrisfetch/LICENSE`

#### Local Build with KISS
```bash
export KISS_PATH="/path/to/ferrisfetch/packaging/kiss:$KISS_PATH"
kiss build ferrisfetch
kiss install ferrisfetch
```

#### KISS Community Repository Submission
1. Fork and clone `kiss-community/community`.
2. Create branch `ferrisfetch`.
3. Place package files in `community/ferrisfetch/` (`build`, `version`, `sources`, `checksums`, `depends`).
4. Commit with message `ferrisfetch: new package at 0.9.8`.
5. Open a pull request against `kiss-community/community:main`.

---

## Release Checklist for Package Maintainers

1. Tag the release: `git tag -a v0.9.8 -m "Release v0.9.8"` and push tags to GitHub.
2. Generate source archive checksum:
   ```bash
   curl -sL https://github.com/kk376/ferrisfetch/archive/refs/tags/v0.9.8.tar.gz | sha256sum
   ```
3. Update version strings and `sha256` checksums across `PKGBUILD`, `.SRCINFO`, `ferrisfetch.spec`, `build.sh`, `default.nix`, `template`, `APKBUILD`, `ebuild`, `ferrisfetch.rb`, and `template.py`.
4. Trigger GitHub release artifacts and update distribution package feeds.


