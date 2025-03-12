{
  description = "Qobuz player";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { 
    self, 
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; config.allowUnfree = true; };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            pkgs.gst_all_1.gstreamer
            pkgs.gst_all_1.gst-plugins-base
            pkgs.glib
            pkgs.pkg-config
            pkgs.cargo-machete
            pkgs.cargo-outdated
            pkgs.sqlx-cli
          ];

          shellHook =
            ''
              export DATABASE_URL="sqlite:///tmp/data.db";
              export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            '';
        };
      }
    );
}
