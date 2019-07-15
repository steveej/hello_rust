with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "env";
  buildInputs = [
    zsh
    rustup
    pkgconfig
    openssl
  ];
}
