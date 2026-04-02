{
  description = "metamorph - model format conversion for local-first AI runtimes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    keel = {
      url = "github:spoke-sh/keel";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, keel }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        isLinux = pkgs.stdenv.isLinux;
        isDarwin = pkgs.stdenv.isDarwin;
        keelPkg = keel.packages.${system}.default;

        metamorph = pkgs.callPackage ./nix/metamorph.nix {
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
        };
      in {
        packages = {
          metamorph = metamorph;
          keel = keelPkg;
          default = metamorph;
        };

        checks = {
          inherit metamorph;
        };

        devShells.default = pkgs.mkShell {
          packages = [
            rust
            keelPkg
            pkgs.just
            pkgs.cargo-nextest
            pkgs.cargo-llvm-cov
            pkgs.pkg-config
          ] ++ pkgs.lib.optionals isLinux [
            pkgs.mold
          ];

          shellHook = ''
            export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/metamorph"
          '' + pkgs.lib.optionalString isDarwin ''
            export TMPDIR=/var/tmp
          '' + pkgs.lib.optionalString isLinux ''
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
          '';
        };
      });
}
