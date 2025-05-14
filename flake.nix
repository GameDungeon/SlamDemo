{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    cargo-v5.url = "github:vexide/cargo-v5";
    utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      utils,
      cargo-v5,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        cargo-v5' = cargo-v5.packages.${system}.default;
      in
      {
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              # Python
              (python3.withPackages (
                python-pkgs: with python-pkgs; [
                  nicegui
                  plotly
                  websockets
                  pyserial
                  numpy
                ]
              ))

              cargo-v5'
              pkgs.cargo-binutils
              (pkgs.rust-bin.nightly."2025-01-01".default.override {
                extensions = [
                  "rust-analyzer"
                  "rust-src"
                  "clippy"
                  "llvm-tools"
                ];
              })

              pkg-config
              fontconfig

              sxiv

              arduino-ide
              arduino
            ];
          };
      }
    );
}
