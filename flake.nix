{
  description = "nix dev environment";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
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
        devShells.default = mkShell rec {
          nativeBuildInputs = [
            pkg-config
          ];
          buildInputs = [
            nil
            rust-analyzer
            rust-bin.stable.latest.default
            udev
            alsa-lib
            vulkan-loader
            xorg.libX11
            xorg.libXrandr
            xorg.libXcursor
            xorg.libXi
            xorg.libXtst
            libxkbcommon
            wayland
            openssl
          ];
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
      }
    );
}
