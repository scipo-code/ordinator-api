{ 

  description = "Ordinator Scheduling System";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }: {
    flake-utils.lib.eachDefaultSystem = (system:
      let 
        pkgs = import nixpkgs {
          inherit system;
          overlays = [];
        };
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          pandas
          openpyxl
        ]);
      in {
        devShell = pkgs.mkShell {
          buildInputs = [
            pythonEnv
            nixpkgs.legacyPackages.x86_64-linux.git
            nixpkgs.legacyPackages.x86_64-linux.just
            nixpkgs.legacyPackages.x86_64-linux.dust
          ];
        };
      });
    };
}
