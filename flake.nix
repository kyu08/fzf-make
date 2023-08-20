{
  description = "fzf-make is the command line tool that execute make target using fuzzy finder with preview window.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.05";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
        in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            bat
            cargo
            gnumake
          ];
        };

        packages = rec {
          fzf-make = pkgs.rustPlatform.buildRustPackage {
            pname = "fzf-make";
            version = "0.7.0";

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          
            buildInputs = [ pkgs.bat ];
          };
          default = fzf-make;
        };
    });
}
