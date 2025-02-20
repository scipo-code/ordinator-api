{
  description = "A flake for the ordinator-scheduler-frontend repo";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }: 
    flake-utils.lib.eachDefaultSystem (system:
      let 
        pkgs = import nixpkgs {
          inherit system;
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # pkgs.deno    # for deno
            pkgs.nodejs  # for vite frontend development
            pkgs.nodePackages.typescript-language-server # typescript language server
          
          ];
        };
    });
}
