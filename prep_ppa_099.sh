#!/usr/bin/env bash
set -euo pipefail

WORKDIR="/home/kk376/.gemini/antigravity-cli/brain/2433ce32-3779-4aa4-a7c5-dc32d1f99783/scratch/ppa_build_0.9.9"
rm -rf "$WORKDIR"
mkdir -p "$WORKDIR"

cd /home/kk376/code/ferrisfetch

# Export clean git archive for 0.9.9
git archive --format=tar.gz --prefix=ferrisfetch-0.9.9/ -o "$WORKDIR/ferrisfetch_0.9.9.orig.tar.gz" HEAD

cd "$WORKDIR"
tar -xzf ferrisfetch_0.9.9.orig.tar.gz
cd ferrisfetch-0.9.9

# Vendor crates for offline Launchpad builds
cargo vendor vendor > .cargo-config.toml
mkdir -p .cargo
cp .cargo-config.toml .cargo/config.toml

# Normalize vendored crates for Ubuntu Noble Cargo 1.75.0 compatibility
find vendor/ -name "Cargo.toml" -type f -exec sed -i 's/edition = "2024"/edition = "2021"/g' {} +
find vendor/ -name "Cargo.toml" -type f -exec sed -i '/rust-version/d' {} +

# Normalize .cargo-checksum.json preserving "package" hash and setting "files": {}
python3 -c '
import glob, json

files = glob.glob("vendor/**/.cargo-checksum.json", recursive=True)
assert len(files) > 0, "No checksum files found"
for f in files:
    with open(f, "r") as fp:
        data = json.load(fp)
    pkg_hash = data.get("package", None)
    new_data = {"package": pkg_hash, "files": {}}
    with open(f, "w") as fp:
        json.dump(new_data, fp)
print(f"Normalized {len(files)} vendored .cargo-checksum.json files.")
'

# Re-create orig tarball with vendored dependencies
cd "$WORKDIR"
rm -rf ferrisfetch_0.9.9.orig.tar.gz
tar -czf ferrisfetch_0.9.9.orig.tar.gz ferrisfetch-0.9.9

cd ferrisfetch-0.9.9
mkdir -p debian/source
cp /home/kk376/code/ferrisfetch/packaging/debian/* debian/ 2>/dev/null || true
echo "3.0 (quilt)" > debian/source/format

# Build unsigned full source package (-sa)
debuild -S -d -sa -us -uc

echo "PPA 0.9.9 package prepared in $WORKDIR."
ls -la "$WORKDIR"
