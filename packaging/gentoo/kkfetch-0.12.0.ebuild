# Copyright 2026 Gentoo Authors
# Distributed under the terms of the MIT License

EAPI=8

CRATES=""

inherit cargo

DESCRIPTION="Fast, lightweight Linux system information fetch tool written in Rust"
HOMEPAGE="https://github.com/kk376/kkfetch"
SRC_URI="https://github.com/kk376/${PN}/archive/refs/tags/v${PV}.tar.gz -> ${P}.tar.gz
	$(cargo_crate_uris)"

LICENSE="MIT"
SLOT="0"
KEYWORDS="~amd64 ~arm64 ~x86"

src_install() {
	cargo_src_install

	dodoc README.md
	doman docs/kkfetch.1
	newbashcomp completions/kkfetch.bash kkfetch
	insinto /usr/share/zsh/site-functions
	newins completions/_kkfetch _kkfetch
	insinto /usr/share/fish/vendor_completions.d
	newins completions/kkfetch.fish kkfetch.fish
}
