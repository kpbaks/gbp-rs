{
  description = "gbp-rs";
  inputs = {
    # wgsl_analyzer.url = "github:wgsl-analyzer/wgsl-analyzer";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    # wgsl_analyzer,
  } @ inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import inputs.nixpkgs {inherit system;};
      # wgsl-analyzer-pkgs = import inputs.wgsl_analyzer {inherit system;};
      bevy-deps = with pkgs; [
        udev
        alsa-lib
        vulkan-loader
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
        libxkbcommon
        wayland
        # wgsl-analyzer-pkgs.wgsl_analyzer
        # wgsl_analyzer.packages.${system}
        # wgsl_analyzer.outputs.packages.${system}.default
      ];
      cargo-subcommands = with pkgs; [
        cargo-bloat
        cargo-expand
        cargo-info
        cargo-outdated
        cargo-show-asm

        #   # cargo-profiler
        #   # cargo-feature
      ];
      rust-deps = with pkgs;
        [
          rustup
          taplo # TOML formatter and LSP
          bacon
          mold # A Modern Linker
          clang # For linking
        ]
        ++ cargo-subcommands;
    in
      with pkgs; {
        formatter.${system} = pkgs.alejandra;
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          buildInputs = [just d2] ++ bevy-deps ++ rust-deps;

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
      });
}
