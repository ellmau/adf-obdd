{
  description = "basic rust flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-21.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    gitignoresrc = {
      url = "github:hercules-ci/gitignore.nix";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, flake-compat, gitignoresrc, rust-overlay, ... }@inputs:
    {
      #overlay = import ./nix { inherit gitignoresrc; };
    } // (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay)];
        };
      in
      rec {
        devShell =
          pkgs.mkShell {
            buildInputs = [
              pkgs.rust-bin.stable.latest.default
              pkgs.rust-analyzer
              pkgs.cargo-audit
              pkgs.cargo-license
            ];
          };
      }
    ));
}
