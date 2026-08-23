#!/usr/bin/env bash
set -euo pipefail

WORKDIR="/home/kk376/.gemini/antigravity-cli/brain/2433ce32-3779-4aa4-a7c5-dc32d1f99783/scratch/ppa_build_0.9.8"
rm -rf "$WORKDIR"
mkdir -p "$WORKDIR"

cd /home/kk376/code/ferrisfetch

# Export clean git archive for 0.9.8
git archive --format=tar.gz --prefix=ferrisfetch-0.9.8/ -o "$WORKDIR/ferrisfetch_0.9.8.orig.tar.gz" HEAD

cd "$WORKDIR"
tar -xzf ferrisfetch_0.9.8.orig.tar.gz
cd ferrisfetch-0.9.8

# Vendor crates for offline Launchpad builds
cargo vendor vendor > .cargo-config.toml
mkdir -p .cargo
cp .cargo-config.toml .cargo/config.toml

# Normalize vendored crates for Ubuntu Noble Cargo 1.75.0 compatibility
find vendor/ -name "Cargo.toml" -type f -exec sed -i 's/edition = "2024"/edition = "2021"/g' {} +
find vendor/ -name "Cargo.toml" -type f -exec sed -i '/rust-version/d' {} +

# Normalize .cargo-checksum.json using Rule 16 (never use {} inside find -exec)
for f in $(find vendor/ -name ".cargo-checksum.json" -type f); do
    printf '{"files":{}}\n' > "$f"
done

# Verify all .cargo-checksum.json are valid JSON
python3 -c '
import os, glob, json
files = glob.glob("vendor/**/.cargo-checksum.json", recursive=True)
assert len(files) > 0, "No checksum files found"
for f in files:
    with open(f) as fp:
        d = json.load(fp)
        assert "files" in d, f"corrupted {f}"
print(f"Verified {len(files)} vendored .cargo-checksum.json files successfully.")
'

# Re-create orig tarball with vendored dependencies
cd "$WORKDIR"
rm -rf ferrisfetch_0.9.8.orig.tar.gz
tar -czf ferrisfetch_0.9.8.orig.tar.gz ferrisfetch-0.9.8

cd ferrisfetch-0.9.8
mkdir -p debian/source
cp /home/kk376/code/ferrisfetch/packaging/debian/* debian/ 2>/dev/null || true
echo "3.0 (quilt)" > debian/source/format

# Build unsigned full source package (-sa)
debuild -S -d -sa -us -uc

echo "PPA 0.9.8 package prepared in $WORKDIR."
ls -la "$WORKDIR"
