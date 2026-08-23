class Ferrisfetch < Formula
  desc "A fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust"
  homepage "https://github.com/kk376/ferrisfetch"
  url "https://github.com/kk376/ferrisfetch/archive/refs/tags/v0.10.1.tar.gz"
  sha256 "0ac74ec930a9769792f3a1eceaf64d6ce1e381f855bcceea6782b2fd50b607be"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    bash_completion.install "completions/ferrisfetch.bash" => "ferrisfetch"
    zsh_completion.install "completions/_ferrisfetch"
    fish_completion.install "completions/ferrisfetch.fish"
  end

  test do
    assert_match "ferrisfetch", shell_output("#{bin}/ferrisfetch --version")
  end
end
