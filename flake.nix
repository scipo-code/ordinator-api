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
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          pandas
          openpyxl
        ]);
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo-cross
            pkgs.rust-bin.stable.latest.default
            pythonEnv
            pkgs.git
            pkgs.helix
            pkgs.zellij
            pkgs.nushell
            pkgs.jq
            pkgs.openssl_3_3
            pkgs.clang
            pkgs.libxlsxwriter
            pkgs.pkg-config
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
