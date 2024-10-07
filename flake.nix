{
  description = "A simple project with a devShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils,... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ ];
        };
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          pandas
          openpyxl
        ]);
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pythonEnv
            pkgs.git
            pkgs.helix
            pkgs.zellij
            pkgs.nushell
            pkgs.jq
            pkgs.openssl
            pkgs.clang
            pkgs.libxlsxwriter
            pkgs.pkg-config
            pkgs.python312Packages.python-lsp-server
            pkgs.rustup
            pkgs.rust-analyzer
          ];
        
        };
        packages.default = pkgs.buildRustPackage {
          pname = "ordinator";
          version = "1.0.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          buildInputs = [
              pkgs.openssl_3_3
              pkgs.pkg-config
          ];
          nativeBuildInputs = [
              pkgs.openssl_3_3
              pkgs.pkg-config
          ];
        };
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      });
} 
