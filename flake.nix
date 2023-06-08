{
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
        apps = rec {
          adf-bdd = flake-utils.lib.mkApp {
            drv = packages.adf-bdd;
            exePath = "/bin/adf-bdd";
          };
        };
        packages = rec {
          adf-bdd = platform.buildRustPackage {
            pname = "adf-bdd";
            version = "0.3.0-dev";
            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "adf-bdd, Abstract Dialectical Frameworks solved by Binary Decision Diagrams; developed in Dresden";
              homepage = "https://github.com/ellmau/adf-obdd";
              license = [pkgs.lib.licenses.mit];
            };

            nativeBuildInputs = with platform; [
              cargoBuildHook
              cargoCheckHook
            ];
            buildAndTestSubdir = "bin";
          };
          adf_bdd = platform.buildRustPackage {
            pname = "adf_bdd";
            version = "0.3.0";
            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "adf-bdd, Abstract Dialectical Frameworks solved by Binary Decision Diagrams; developed in Dresden";
              homepage = "https://github.com/ellmau/adf-obdd";
              license = [pkgs.lib.licenses.mit];
            };

            nativeBuildInputs = with platform; [
              cargoBuildHook
              cargoCheckHook
            ];
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
            ++ (notOn ["aarch64-darwin" "x86_64-darwin"] [pkgs.kcov pkgs.cargo-kcov pkgs.gnuplot])
            ++ (notOn ["aarch64-linux" "aarch64-darwin" "i686-linux"] [pkgs.cargo-tarpaulin])
            ++ (notOn ["aarch64-darwin" "x86_64-darwin"] [pkgs.valgrind]);
        };
      };
    };
}
