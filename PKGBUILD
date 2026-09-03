# Maintainer: Kushagra Kumar (kk376) <kk376@archlinux.local>
pkgname=kkfetch
pkgver=0.13.0
pkgrel=1
pkgdesc="A fast, lightweight Linux system information fetch tool written in Rust"
arch=('x86_64' 'aarch64')
url="https://github.com/kk376/kkfetch"
license=('MIT')
depends=('glibc')
makedepends=('cargo' 'rust')
source=("$pkgname-$pkgver.tar.gz::https://github.com/kk376/kkfetch/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

check() {
    cd "$pkgname-$pkgver"
    cargo test --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 "completions/$pkgname.bash" "$pkgdir/usr/share/bash-completion/completions/$pkgname"
    install -Dm644 "completions/_$pkgname" "$pkgdir/usr/share/zsh/site-functions/_$pkgname"
    install -Dm644 "completions/$pkgname.fish" "$pkgdir/usr/share/fish/vendor_completions.d/$pkgname.fish"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
