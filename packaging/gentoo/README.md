# Gentoo Linux Packaging for FerrisFetch

This directory contains the Gentoo Linux ebuild recipe and package metadata for `app-misc/ferrisfetch` (or `sys-apps/ferrisfetch`), conforming to Gentoo GURU and Portage standards.

## Files

- `ferrisfetch-0.10.0.ebuild`: Portage ebuild using `EAPI=8`, `cargo.eclass`, and `shell-completion.eclass`.
- `metadata.xml`: Package metadata with maintainer contact and upstream repository.

## Installation / Usage in Local Overlay

1. **Create local overlay structure** (if not already existing):
   ```bash
   mkdir -p /var/db/repos/localrepo/app-misc/ferrisfetch
   ```

2. **Copy the ebuild and metadata**:
   ```bash
   cp ferrisfetch-0.10.0.ebuild metadata.xml /var/db/repos/localrepo/app-misc/ferrisfetch/
   ```

3. **Generate Manifest with cargo checksums**:
   ```bash
   cd /var/db/repos/localrepo/app-misc/ferrisfetch
   ebuild ferrisfetch-0.10.0.ebuild manifest
   ```

4. **Test build locally**:
   ```bash
   ebuild ferrisfetch-0.10.0.ebuild clean compile
   ebuild ferrisfetch-0.10.0.ebuild install
   ```

5. **Emerge ferrisfetch**:
   ```bash
   emerge --ask app-misc/ferrisfetch
   ```

## Contributing to GURU / Gentoo Main Tree

To submit `ferrisfetch` to the [Gentoo GURU overlay](https://wiki.gentoo.org/wiki/Project:GURU):

1. Fork the `gentoo/guru` repository on GitHub / GitLab.
2. Clone your fork and create a new branch: `git checkout -b ferrisfetch-0.10.0`
3. Add the package to `app-misc/ferrisfetch/`.
4. Run `pkgcheck scan` and `repoman full` to verify Gentoo QA compliance.
5. Commit with standard Gentoo commit message:
   ```bash
   git commit -m "app-misc/ferrisfetch: new package, add 0.10.0"
   ```
