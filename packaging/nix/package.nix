{ lib
, rustPlatform
, fetchFromGitHub
, installShellFiles
, src ? null
}:

rustPlatform.buildRustPackage rec {
  pname = "kkfetch";
  version = "0.11.7";

  src = if src != null then src else fetchFromGitHub {
    owner = "kk376";
    repo = "kkfetch";
    rev = "v${version}";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  nativeBuildInputs = [ installShellFiles ];

  postInstall = ''
    installShellCompletion --cmd kkfetch \
      --bash completions/kkfetch.bash \
      --fish completions/kkfetch.fish \
      --zsh completions/_kkfetch
  '';

  meta = with lib; {
    description = "Fast, lightweight Linux system information fetch tool written in Rust";
    homepage = "https://github.com/kk376/kkfetch";
    license = licenses.mit;
    maintainers = [ "Kushagra Kumar (kk376)" ];
    mainProgram = "kkfetch";
    platforms = platforms.linux;
  };
}
