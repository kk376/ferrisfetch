TERMUX_PKG_NAME="kkfetch"
TERMUX_PKG_HOMEPAGE="https://github.com/kk376/kkfetch"
TERMUX_PKG_DESCRIPTION="A fast, lightweight Linux and Android system information fetch CLI written in Rust"
TERMUX_PKG_LICENSE="MIT"
TERMUX_PKG_MAINTAINER="Kushagra Kumar (kk376) <kk376@users.noreply.github.com>"
TERMUX_PKG_VERSION="0.12.0"
TERMUX_PKG_SRCURL="https://github.com/kk376/kkfetch/archive/refs/tags/v${TERMUX_PKG_VERSION}.tar.gz"
TERMUX_PKG_SHA256=b07371f86878400c474b17bf4c16c8b98a6aabc9dd5181271a3c773f8589c679
TERMUX_PKG_AUTO_UPDATE=true
TERMUX_PKG_BUILD_IN_SRC=true

termux_step_make() {
	termux_setup_rust
	cargo build --jobs $TERMUX_PKG_MAKE_PROCESSES --target $CARGO_TARGET_NAME --release
}

termux_step_make_install() {
	install -Dm755 target/${CARGO_TARGET_NAME}/release/kkfetch $TERMUX_PREFIX/bin/kkfetch
	install -Dm644 docs/kkfetch.1 $TERMUX_PREFIX/share/man/man1/kkfetch.1
	install -Dm644 completions/kkfetch.bash $TERMUX_PREFIX/share/bash-completion/completions/kkfetch
	install -Dm644 completions/_kkfetch $TERMUX_PREFIX/share/zsh/site-functions/_kkfetch
	install -Dm644 completions/kkfetch.fish $TERMUX_PREFIX/share/fish/vendor_completions.d/kkfetch.fish
}
