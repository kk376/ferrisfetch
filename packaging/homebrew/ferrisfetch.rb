class Ferrisfetch < Formula
  desc "Fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust"
  homepage "https://github.com/kk376/ferrisfetch"
  url "https://github.com/kk376/ferrisfetch/releases/download/v0.11.4/ferrisfetch-0.11.4-x86_64-unknown-linux-gnu.tar.gz"
  sha256 "3b9686d2181937b9e83cc2d732ca94adb5aea1e3f03a54d193c6ef99ee48cf86"
  license "MIT"
  version "0.11.4"

  def install
    bin.install "ferrisfetch"
    man1.install "man/ferrisfetch.1" if File.exist?("man/ferrisfetch.1")
    bash_completion.install "completions/ferrisfetch.bash" => "ferrisfetch" if File.exist?("completions/ferrisfetch.bash")
    zsh_completion.install "completions/_ferrisfetch" => "_ferrisfetch" if File.exist?("completions/_ferrisfetch")
    fish_completion.install "completions/ferrisfetch.fish" if File.exist?("completions/ferrisfetch.fish")
  end

  test do
    assert_match "ferrisfetch 0.11.4", shell_output("#{bin}/ferrisfetch --version")
  end
end
