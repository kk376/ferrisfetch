class Ferrisfetch < Formula
  desc "Fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust"
  homepage "https://github.com/kk376/ferrisfetch"
  url "https://github.com/kk376/ferrisfetch/releases/download/v0.11.3/ferrisfetch-0.11.3-x86_64-unknown-linux-gnu.tar.gz"
  sha256 "26718bdd68cb913ff44978ec406bcc3903fd1afda258bc32e91babb0c4d27a1f"
  license "MIT"
  version "0.11.3"

  def install
    bin.install "ferrisfetch"
    man1.install "man/ferrisfetch.1" if File.exist?("man/ferrisfetch.1")
    bash_completion.install "completions/ferrisfetch.bash" => "ferrisfetch" if File.exist?("completions/ferrisfetch.bash")
    zsh_completion.install "completions/_ferrisfetch" => "_ferrisfetch" if File.exist?("completions/_ferrisfetch")
    fish_completion.install "completions/ferrisfetch.fish" if File.exist?("completions/ferrisfetch.fish")
  end

  test do
    assert_match "ferrisfetch 0.11.3", shell_output("#{bin}/ferrisfetch --version")
  end
end
