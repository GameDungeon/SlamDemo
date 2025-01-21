{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:

    (flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        libPath =
          with pkgs;
          lib.makeLibraryPath [
            libGL
            libxkbcommon
            wayland
          ];
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.nightly.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
                "clippy"
              ];

              targets = [
                "x86_64-unknown-linux-gnu"
                "wasm32-unknown-unknown"
              ];
            })

            pkg-config
            fontconfig
            wayland

            trunk
          ];

          LD_LIBRARY_PATH = libPath;
        };
      }
    ));
}
