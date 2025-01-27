{
  description = "The ordinator-api flake.nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          pandas
          openpyxl
          matplotlib
        ]);
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.linuxKernel.packages.linux_zen.perf
            pkgs.cargo-cross
            pkgs.cargo-release
            pkgs.rust-bin.beta.latest.default
            pythonEnv
            pkgs.git
            pkgs.zellij
            (pkgs.gnuplot.override {
              withLua = true;
              lua = pkgs.lua;
            })
            pkgs.nushell
            pkgs.jq
            pkgs.openssl_3_3
            pkgs.clang
            pkgs.libxlsxwriter
            pkgs.pkg-config
            pkgs.libunwind
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
      });
} 
