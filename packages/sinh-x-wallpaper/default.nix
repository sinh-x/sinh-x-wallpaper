{
  # Snowfall Lib provides a customized `lib` instance with access to your flake's library
  # as well as the libraries available from your flake's inputs.
  # You also have access to your flake's inputs.

  # The namespace used for your flake, defaulting to "internal" if not set.

  # All other arguments come from NixPkgs. You can use `pkgs` to pull packages or helpers
  # programmatically or you may add the named attributes as arguments here.
  pkgs,
  ...
}:
pkgs.rustPlatform.buildRustPackage {
  pname = "sinh-x-wallpaper";
  version = "0.3.0";
  src = ../..;
  cargoHash = "sha256-3lG/J6iAlU/Xk/MG8HD9im7qalhO+jHxhlxJ8TTICL8=";
  buildInputs = with pkgs; [
    cargo
    rustc
    pkg-config
    openssl
    rustfmt
    clippy
  ];
  nativeBuildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.pkg-config
    pkgs.openssl
  ];
}
