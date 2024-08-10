{
  description = "Sinh-x-wallpaper";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        sinh-x-wallpaper = pkgs.rustPlatform.buildRustPackage {
          pname = "sinh-x-wallpaper";
          version = "0.3.0";
          src = ./.;
          cargoHash = "sha256-3lG/J6iAlU/Xk/MG8HD9im7qalhO+jHxhlxJ8TTICL8=";
          buildInputs = with pkgs; [
            cargo
            rustc
            pkg-config
            openssl
            rustfmt
            clippy
          ];
          nativeBuildInputs = [pkgs.cargo pkgs.rustc pkgs.pkg-config pkgs.openssl];
        };
      in {
        defaultPackage = sinh-x-wallpaper;

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            pkg-config
            openssl
            rustfmt
            clippy
          ];
          shellHook = ''
            exec fish
          '';
        };
      }
    );
}
