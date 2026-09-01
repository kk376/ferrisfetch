class Kkfetch < Formula
  desc "Fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust"
  homepage "https://github.com/kk376/kkfetch"
  url "https://github.com/kk376/kkfetch/releases/download/v0.12.0/kkfetch-0.12.0-x86_64-unknown-linux-gnu.tar.gz"
  sha256 "0f0df08111849d41ba40d1a441894553db6122be1147d36f271670f57b704803"
  license "MIT"
  version "0.12.0"

  def install
    bin.install "kkfetch"
    man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
    bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
    zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
    fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
  end

  test do
    assert_match "kkfetch 0.12.0", shell_output("#{bin}/kkfetch --version")
  end
end
