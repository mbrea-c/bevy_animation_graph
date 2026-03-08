{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      linkedLibraries = [
        pkgs.udev
        pkgs.alsa-lib
        pkgs.vulkan-loader
        # X11
        pkgs.libx11
        pkgs.libxcursor
        pkgs.libxi
        pkgs.libxrandr
        # Wayland
        pkgs.libxkbcommon
        pkgs.wayland
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          toolchain

          pkgs.pkg-config
          pkgs.ldtk

          # We want the unwrapped version, "rust-analyzer" (wrapped) comes with nixpkgs' toolchain
          pkgs.rust-analyzer-unwrapped
        ]
        ++ linkedLibraries;

        RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath linkedLibraries;
      };
    };
}
