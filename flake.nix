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
          buildInputs = with pkgs; [ clang llvm_16 rustup glib gcc ];

          RUSTC_VERSION = overrides.toolchain.channel;

          LIBCLANG_PATH =
            pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

          shellHook = ''
            export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
            export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/

            # Set Git configuration
            git config --global user.name "wangfiox"
            git config --global user.email "wangfiox@gmail.com"
            git config --global core.editor "nvim"
          '';

          # 这是一个 lambda
          RUSTFLAGS = builtins.map (a: "-L ${a}/lib") [
            # add libraries here (e.g. pkgs.libvmi)
          ];

          LD_LIBRARY_PATH = libPath;

          BINDGEN_EXTRA_CLANG_ARGS = (builtins.map (a: ''-I"${a}/include"'') [
            # add dev libraries here (e.g. pkgs.libvmi.dev)
            pkgs.glibc.dev
          ]) ++ [
            ''
              -I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
            ''-I"${pkgs.glib.dev}/include/glib-2.0"''
            "-I${pkgs.glib.out}/lib/glib-2.0/include/"
          ];
        };
      });
}
