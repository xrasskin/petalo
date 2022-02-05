# The core functionality is provided here using flakes. Legacy support for
# `nix-shell` is provided by a wrapper in `shell.nix`.

# TODO HDF5
# TODO bindgen
# TODO PyO3
# TODO nixGL

{
  description = "PET image reconstruction";

  inputs = {

    # Version pinning is managed in flake.lock. Upgrading can be done with
    # something like
    #
    #    nix flake lock --update-input nixpkgs

    nixpkgs     .url = "github:nixos/nixpkgs/nixos-21.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils .url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:

    # Option 1: try to support each default system
    flake-utils.lib.eachDefaultSystem # NB Some packages in nixpkgs are not supported on some systems

    # Option 2: try to support selected systems
    # flake-utils.lib.eachSystem ["x86_64-linux" "i686-linux" "aarch64-linux" "x86_64-darwin"]
      (system:

        let pkgs = import nixpkgs {
              inherit system;
              # Any overlays you need can go here
              overlays = [ (import rust-overlay) ];
            };
            # ----- Conditional inclusion ----------------------------------------------------
            nothing = pkgs.coreutils;
            linux      = drvn: if pkgs.stdenv.isLinux  then drvn else nothing;
            darwin     = drvn: if pkgs.stdenv.isDarwin then drvn else nothing;
            linuxList  = list: if pkgs.stdenv.isLinux  then list else [];
            darwinList = list: if pkgs.stdenv.isDarwin then list else [];
            darwinStr  = str:  if pkgs.stdenv.isDarwin then str  else "";

            # ----- Darwin-specific ----------------------------------------------------------
            darwin-frameworks = pkgs.darwin.apple_sdk.frameworks;

            # ----- Rust versions and extensions ----------------------------------
            extras = {
              extensions = [

                # Uncomment the extensions you want to enable, in the list below.
                # Some are enabled by default.

                # A history of available extensions is published on
                #
                #    https://rust-lang.github.io/rustup-components-history
                #
                # but this appears to contradict reality. To get an error message
                # which lists all available components, uncomment the following
                # line

                #  "does-not-exist" # Uncomment this line to generate list of extensions

                # Extensions which might be available. Uncomment those you want to
                # enable. Some are enabled by default.

                # "cargo"                    # enabled by default
                # "clippy"                   # enabled by default
                # "clippy-preview"           # seems to give the same version as clippy
                # "llvm-tools-preview"
                # "miri"
                # "miri-preview"
                # "reproducible-artifacts"
                # "rls"
                # "rls-preview"              # seems to give the same version as rls
                # "rust"
                # "rust-analysis"
                # "rust-analyzer-preview"
                # "rust-docs"
                # "rust-mingw"
                # "rust-src"
                # "rust-std"
                # "rustc"
                # "rustc-dev"
                # "rustc-docs"
                # "rustfmt"
                # "rustfmt-preview"
              ];
              #targets = [ "arg-unknown-linux-gnueabihf" ];
            };

            # If you already have a rust-toolchain file for rustup, you can simply
            # use fromRustupToolchainFile to get the customized toolchain
            # derivation.
            rust-tcfile  = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;

            rust-latest  = pkgs.rust-bin.stable .latest      .default;
            rust-beta    = pkgs.rust-bin.beta   ."2022-01-25".default;
            rust-nightly = pkgs.rust-bin.nightly."2022-01-25".default;
            rust-stable  = pkgs.rust-bin.stable ."1.58.1"    .default;

            # Rust system to be used in buldiInputs. Choose between
            # latest/beta/nightly/stable on the next line
            rust = rust-stable.override extras;

        in
          {
            devShell = pkgs.mkShell {
              name = "petalorust";
              buildInputs = [
                rust
                pkgs.rust-analyzer
                pkgs.just

                # HDF5
                pkgs.hdf5

                # Needed for compilation to succeed on Macs
                (darwin darwin-frameworks.AppKit)
                (darwin darwin-frameworks.CoreText)

                # python
                (pkgs.python39.withPackages (ps: [
                  ps.numpy
                  ps.cffi
                  ps.jupyter
                  ps.matplotlib
                  ps.pytest
                  ps.h5py
                  ps.scikit-learn
                  ps.docopt
                ]))

              ];
              packages = [
              ];
              HDF5_DIR = pkgs.symlinkJoin { name = "hdf5"; paths = [ pkgs.hdf5 pkgs.hdf5.dev ]; };
              shellHook =
                ''
                  export PS1="rust devshell> "
                  alias foo='cowsay Foo'
                  alias bar='exa -l | lolcat'
                  alias baz='cowsay What is the difference between buildIntputs and packages? | lolcat'
                '';
            };
          }
      );
}
