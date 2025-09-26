{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    utils,
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs { inherit system; };
        nativeBuildInputs' = with pkgs; [
          pkg-config
        ];
        buildInputs' = with pkgs; [
          openssl
          alsa-lib
        ];
      in rec {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [] ++ nativeBuildInputs' ++ buildInputs';

          # shellHook = ''
          #   export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
          #   export MOSQUITTO_PLUGIN_CLANG_EXTRA_ARGS="-I ${pkgs.mosquitto.dev}/include"
          # '';
        };
      }
    );
}
