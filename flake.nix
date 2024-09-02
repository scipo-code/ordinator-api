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
        devShell.x86_64-linux.default = pkgs.mkShell {
          buildInputs = [
            pythonEnv
            pkgs.git
            pkgs.just
            pkgs.dust
          ];
        };
      });
    };
}
