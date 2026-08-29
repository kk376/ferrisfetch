class Ferrisfetch < Formula
  desc "A fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust"
  homepage "https://github.com/kk376/ferrisfetch"
  url "https://github.com/kk376/ferrisfetch/archive/refs/tags/v0.11.1.tar.gz"
  sha256 "2a870860a0d0bc3e6b1c4f3fe8509f133148b8d3faa9eda66aa3d62bb0198846"
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
