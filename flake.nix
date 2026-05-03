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
        bun = pkgs.bun;
      in
      {
        devShells = {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              bun
              pkgs.nodePackages.typescript
            ];

            shellHook = ''
              echo "nestvim development environment"
              echo "Run 'bun install' to install dependencies"
              echo "Run 'bun run start' to start the editor"
            '';
          };
        };

        packages = {
          default = pkgs.stdenv.mkDerivation {
            pname = "nestvim";
            version = "0.1.0";
            src = ./.;

            nativeBuildInputs = [ bun ];

            buildPhase = ''
              runHook preBuild
              bun install
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              mkdir -p $out/bin
              cp -r . $out/
              cat > $out/bin/nestvim << EOF
              #!/bin/sh
              exec ${bun}/bin/bun run $out/src/main.ts "\$@"
              EOF
              chmod +x $out/bin/nestvim
              runHook postInstall
            '';
          };
        };
      }
    );
}
