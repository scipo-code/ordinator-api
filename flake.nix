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
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          pandas
          openpyxl
        ]);
        rustPkgs = pkgs.rustPlatform;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pythonEnv
            (pkgs.rust-bin.stable."1.81.0".default)
            pkgs.git
            pkgs.helix
            pkgs.zellij
            pkgs.nushell
            pkgs.jq
            pkgs.openssl_3_3
            pkgs.clang
            pkgs.libxlsxwriter
            pkgs.pkg-config
            pkgs.python312Packages.python-lsp-server
          ];
          pure = true;
        };
        packages.default = rustPkgs.buildRustPackage {
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
        };
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
         
      });
} 
