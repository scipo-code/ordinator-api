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
            (pkgs.gnuplot.override {
              lua = pkgs.lua;
              withLua = true;
            })
            pkgs.cargo-cross
            pkgs.cargo-release
            pkgs.clang
            pkgs.flamegraph
            pkgs.git
            pkgs.jq
            pkgs.libunwind
            pkgs.libxlsxwriter
            pkgs.linuxKernel.packages.linux_zen.perf
            pkgs.nushell
            pkgs.openssl_3
            pkgs.pkg-config
            pkgs.taplo
            pkgs.rust-bin.nightly.latest.default
                        pkgs.zellij
            pythonEnv

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
              pkgs.openssl_3
              pkgs.pkg-config
          ];
          nativeBuildInputs = [
              pkgs.openssl_3
              pkgs.pkg-config
          ];
        };
      });
} 
