# Gentoo Linux Packaging for FerrisFetch

This directory contains the Gentoo Linux ebuild recipe and package metadata for `app-misc/ferrisfetch` (or `sys-apps/ferrisfetch`), conforming to Gentoo GURU and Portage standards.

## Files

- `ferrisfetch-0.9.9.ebuild`: Portage ebuild using `EAPI=8`, `cargo.eclass`, and `shell-completion.eclass`.
- `metadata.xml`: Package metadata conforming to Gentoo upstream standards.

## Adding to a Local Overlay

1. Create a local overlay category directory:
   ```bash
   mkdir -p /var/db/repos/localrepo/app-misc/ferrisfetch
   ```

2. Copy the ebuild and metadata files:
   ```bash
   cp ferrisfetch-0.9.9.ebuild metadata.xml /var/db/repos/localrepo/app-misc/ferrisfetch/
   ```

3. Generate the Manifest file:
   ```bash
   cd /var/db/repos/localrepo/app-misc/ferrisfetch
   ebuild ferrisfetch-0.9.9.ebuild manifest
   ```

4. Test compilation and installation:
   ```bash
   ebuild ferrisfetch-0.9.9.ebuild clean compile
   ebuild ferrisfetch-0.9.9.ebuild install
   ```

5. Install using `emerge`:
   ```bash
   emerge --ask app-misc/ferrisfetch
   ```

## Gentoo GURU Overlay Submission

For submission to Gentoo's GURU repository:
1. Verify with `pkgcheck`:
   ```bash
   pkgcheck scan
   ```
2. Commit conforming to Gentoo git conventions:
   ```bash
   git add app-misc/ferrisfetch/
   git commit -m "app-misc/ferrisfetch: new package, add 0.9.9"
   ```
