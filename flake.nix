{
  description = "fzf-make is the command line tool that execute make target using fuzzy finder with preview window.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        cargoLock = {
          lockFile = ./Cargo.lock;
          outputHashes = {
            "tree-sitter-just-0.0.1" = "sha256-b42Dt9X0gaHjywb+tahNomGfDx9ZP+roudNuGAhKYPg=";
          };
        };
      in
      rec {
        devShells.default = pkgs.mkShell {
          inputsFrom = [ packages.fzf-make ];
          packages = with pkgs; [ rustfmt clippy typos ];
        };

        formatter = pkgs.nixpkgs-fmt;
        packages = rec {
          fzf-make = pkgs.rustPlatform.buildRustPackage {
            pname = "fzf-make";
            src = ./.;

            inherit (cargoTOML.package) version;
            inherit cargoLock;

            nativeBuildInputs = [ pkgs.makeBinaryWrapper ];
            postInstall =
              let
                runtimeDeps = with pkgs; [ gnugrep gnumake ];
              in
              ''
                wrapProgram $out/bin/fzf-make \
                  --set SHELL ${pkgs.runtimeShell} \
                  --suffix PATH : ${pkgs.lib.makeBinPath runtimeDeps}
              '';
          };
          default = fzf-make;
        };
      });
}
