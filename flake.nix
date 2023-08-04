rec {
  description = "adf-bdd, Abstract Dialectical Frameworks solved by Binary Decision Diagrams; developed in Dresden";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils/flake-utils";
      };
    };
    flake-utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
  };

  outputs = inputs @ {
    self,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.mkFlake {
      inherit self inputs;
      channels.nixpkgs.overlaysBuilder = channels: [rust-overlay.overlays.default];
      outputsBuilder = channels: let
        pkgs = channels.nixpkgs;
        toolchain = pkgs.rust-bin.stable.latest.default;
        platform = pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        };
      in rec {
        packages = let
          cargoMetaBin = (builtins.fromTOML (builtins.readFile ./bin/Cargo.toml)).package;
          cargoMetaLib = (builtins.fromTOML (builtins.readFile ./lib/Cargo.toml)).package;
          meta = {
            inherit description;
            homepage = "https://github.com/ellmau/adf-obdd";
            license = [pkgs.lib.licenses.mit];

            nativeBuildInputs = with platform; [
              cargoBuildHook
              cargoCheckHook
            ];
          };
        in rec {
          adf-bdd = platform.buildRustPackage {
            pname = "adf-bdd";
            inherit (cargoMetaBin) version;
            inherit meta;

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            buildAndTestSubdir = "bin";
          };
          adf_bdd = platform.buildRustPackage {
            pname = "adf_bdd";
            inherit (cargoMetaLib) version;
            inherit meta;

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            buildAndTestSubdir = "lib";
          };
        };
        devShells.default = pkgs.mkShell {
          RUST_LOG = "debug";
          RUST_BACKTRACE = 1;
          shellHook = ''
            export PATH=''${HOME}/.cargo/bin''${PATH+:''${PATH}}
          '';
          buildInputs = let
            notOn = systems:
              pkgs.lib.optionals (!builtins.elem pkgs.system systems);
          in
            [
              toolchain
              pkgs.rust-analyzer
              pkgs.cargo-audit
              pkgs.cargo-license
            ]
            ++ (notOn ["aarch64-darwin" "x86_64-darwin"] [pkgs.kcov pkgs.cargo-kcov pkgs.gnuplot pkgs.valgrind])
            ++ (notOn ["aarch64-linux" "aarch64-darwin" "i686-linux"] [pkgs.cargo-tarpaulin]);
        };
      };
    };
}
