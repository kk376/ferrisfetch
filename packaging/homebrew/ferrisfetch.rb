class Ferrisfetch < Formula
  desc "Fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust"
  homepage "https://github.com/kk376/ferrisfetch"
  url "https://github.com/kk376/ferrisfetch/releases/download/v0.11.2/ferrisfetch-0.11.2-x86_64-unknown-linux-gnu.tar.gz"
  sha256 "5d9e854c7022b48afeefdfb55e51e034956bcad3e8d8479e6bcfa06de48c97a6"
  license "MIT"
  version "0.11.2"

  def install
    bin.install "ferrisfetch"
    man1.install "man/ferrisfetch.1" if File.exist?("man/ferrisfetch.1")
    bash_completion.install "completions/ferrisfetch.bash" => "ferrisfetch" if File.exist?("completions/ferrisfetch.bash")
    zsh_completion.install "completions/_ferrisfetch" => "_ferrisfetch" if File.exist?("completions/_ferrisfetch")
    fish_completion.install "completions/ferrisfetch.fish" if File.exist?("completions/ferrisfetch.fish")
  end

  test do
    assert_match "ferrisfetch 0.11.2", shell_output("#{bin}/ferrisfetch --version")
  end
end
