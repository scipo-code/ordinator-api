{
  description = "A simple project with a devShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          pandas
          openpyxl
        ]);
        rustPkgs = pkgs.rustPlatform;
      in {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            pythonEnv
            git
            helix
            zellij
            nushell
            jq
            openssl_3_3
            clang
            libxlsxwriter
            pkg-config
            python312Packages.python-lsp-server
            rust-bin.beta.latest.default

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
