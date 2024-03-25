{
  description = "fzf-make is the command line tool that execute make target using fuzzy finder with preview window.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      rec
      {
        devShells.default = pkgs.mkShell {
          inputsFrom = [ packages.fzf-make ];
          packages = with pkgs; [ clippy typos ];
        };

        formatter = pkgs.nixpkgs-fmt;
        packages = rec {
          fzf-make = pkgs.rustPlatform.buildRustPackage {
            pname = "fzf-make";
            inherit (cargoTOML.package) version;

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ pkgs.makeBinaryWrapper ];
            postInstall =
              let
                runtimeDeps = with pkgs; [ bat gnugrep gnumake ];
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
