{ lib
, rustPlatform
, fetchFromGitHub
, installShellFiles
, src ? null
}:

rustPlatform.buildRustPackage rec {
  pname = "ferrisfetch";
  version = "0.11.6";

  src = if src != null then src else fetchFromGitHub {
    owner = "kk376";
    repo = "ferrisfetch";
    rev = "v${version}";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  nativeBuildInputs = [ installShellFiles ];

  postInstall = ''
    installShellCompletion --cmd ferrisfetch \
      --bash completions/ferrisfetch.bash \
      --fish completions/ferrisfetch.fish \
      --zsh completions/_ferrisfetch
  '';

  meta = with lib; {
    description = "Fast, lightweight Linux system information fetch tool written in Rust";
    homepage = "https://github.com/kk376/ferrisfetch";
    license = licenses.mit;
    maintainers = [ ];
    mainProgram = "ferrisfetch";
    platforms = platforms.linux;
  };
}
