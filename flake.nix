{
  description = "A Rust application";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacypackages.${system};
        sinh-x-wallpaper = pkgs.rustplatform.buildrustpackage {
          pname = "sinh-x-wallpaper";
          version = "0.1.0";
          src = ./.;
          cargosha256 = "";
          buildinputs = [pkgs.openssl];
          nativeBuildInputs = [pkgs.cargo pkgs.rustc pkgs.pkg-config pkgs.openssl];
          ld_library_path = pkgs.lib.makelibrarypath [
            pkgs.openssl
          ];
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
        };
      }
    );
}
