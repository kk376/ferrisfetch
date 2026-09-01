class Kkfetch < Formula
  desc "Fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust"
  homepage "https://github.com/kk376/kkfetch"
  url "https://github.com/kk376/kkfetch/releases/download/v0.11.7/kkfetch-0.11.7-x86_64-unknown-linux-gnu.tar.gz"
  sha256 "cce5fe5d203f368a110ddac5713c268b66949c6f0c94f8655169148c5663d919"
  license "MIT"
  version "0.11.7"

  def install
    bin.install "kkfetch"
    man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
    bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
    zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
    fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
  end

  test do
    assert_match "kkfetch 0.11.7", shell_output("#{bin}/kkfetch --version")
  end
end
