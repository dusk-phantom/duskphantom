{
  description = "Rust development environment with Clang and LLVM support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
        libPath = pkgs.lib.makeLibraryPath [
          # load external libraries that you need in your rust project here
        ];
      in {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            llvmPackages_16.bintools
            llvmPackages_16.libllvm
            llvmPackages_16.llvm
            llvmPackages_16.stdenv
            clang
            rustup
            glib
            gcc
            libxml2
          ];
          LD_LIBRARY_PATH = libPath;
          RUSTC_VERSION = overrides.toolchain.channel;

          shellHook = ''
            export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
            export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
          '';

          # LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

        };
      });
}
