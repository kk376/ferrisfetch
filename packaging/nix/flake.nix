{
  description = "Fast, lightweight Linux system information fetch tool written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "i686-linux"
        "riscv64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });
    in
    {
      packages = forAllSystems (system: {
        kkfetch = nixpkgsFor.${system}.callPackage ./package.nix {
          src = nixpkgs.lib.cleanSource ../..;
        };
        default = self.packages.${system}.kkfetch;
      });

      apps = forAllSystems (system: {
        kkfetch = {
          type = "app";
          program = "${self.packages.${system}.kkfetch}/bin/kkfetch";
        };
        default = self.apps.${system}.kkfetch;
      });

      devShells = forAllSystems (system: {
        default = nixpkgsFor.${system}.mkShell {
          inputsFrom = [ self.packages.${system}.kkfetch ];
          packages = with nixpkgsFor.${system}; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];
        };
      });

      overlays.default = final: prev: {
        kkfetch = final.callPackage ./package.nix {
          src = final.lib.cleanSource ../..;
        };
      };
    };
}
