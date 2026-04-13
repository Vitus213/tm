{
  description = "tm development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;

        runtimeLibs = with pkgs; [
          alsa-lib
          libGL
          libx11
          libxcursor
          libxi
          libxinerama
          libxkbcommon
          libxrandr
          sqlite
          udev
          vulkan-loader
          wayland
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            pkg-config
            rust-analyzer
            rustc
            rustfmt
          ] ++ runtimeLibs;

          LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;

          shellHook = ''
            export WINIT_UNIX_BACKEND=''${WINIT_UNIX_BACKEND:-wayland}
            echo "tm dev shell ready"
          '';
        };
      });
}
