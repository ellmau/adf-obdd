{
  description = "basic rust flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
    gitignoresrc = {
      url = "github:hercules-ci/gitignore.nix";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, nixpkgs-unstable, flake-utils, gitignoresrc, rust-overlay, ... }@inputs:
    {
      #overlay = import ./nix { inherit gitignoresrc; };
    } // (flake-utils.lib.eachDefaultSystem (system:
      let
        unstable = import nixpkgs-unstable { inherit system; };
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay)];
        };
      in
      rec {
        devShell =
          pkgs.mkShell {
            RUST_LOG = "debug";
            RUST_BACKTRACE = 1;
            buildInputs = [
              pkgs.rust-bin.nightly.latest.rustfmt
              pkgs.rust-bin.stable.latest.default
              pkgs.rust-analyzer
              pkgs.cargo-audit
              pkgs.cargo-license
              pkgs.cargo-tarpaulin
              pkgs.cargo-kcov
              pkgs.valgrind
              pkgs.gnuplot
              pkgs.kcov
            ];
          };
      }
    ));
}
