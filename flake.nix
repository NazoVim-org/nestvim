{
  description = "nestvim - A minimal Vim-like TUI editor";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells = {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              pkgs.rustc
              pkgs.cargo
              pkgs.rustfmt
              pkgs.clippy
            ];

            shellHook = ''
              echo "nestvim development environment"
              echo "Run 'cargo build' to build the project"
              echo "Run 'cargo run -- [file]' to start the editor"
            '';
          };
        };

        packages = {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "nestvim";
            version = "0.1.0";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };
        };
      }
    );
}
