{
  inputs = {
    nixpkgs.url      = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShell = mkShell rec {
          buildInputs = [
            (rust-bin.stable.latest.default.override {
              targets = [ "x86_64-unknown-linux-gnu" "thumbv7em-none-eabihf" ];
            })

            # flashing dependencies
            gcc-arm-embedded
            openocd
          ];

          shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRRAY_PATH:${lib.makeLibraryPath buildInputs}"
          '';
        };
      }
    );
}
