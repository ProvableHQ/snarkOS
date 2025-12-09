{
  description = "snarkOS - A decentralized operating system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      systems,
    }:
    let
      eachSystem = f: nixpkgs.lib.genAttrs (import systems) (system: f system);

      # Common configuration factory that takes pkgs as input
      mkCommonConfig = pkgs: rec {
        # Rust toolchain matching rust-toolchain.toml
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Native build inputs for the project
        nativeBuildInputs = with pkgs; [
          rustToolchain
          curl
          openssl
          zlib
          pkg-config
          clang
          gcc
          lld
          libclang
          git # Required for build.rs (built crate with git2 feature)
        ];

        # Runtime dependencies
        buildInputs = with pkgs; [
          openssl
        ];

        # Environment variables
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        GIT = "${pkgs.git}/bin/git";
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
          pkgs.zlib
          pkgs.openssl
          pkgs.libclang.lib
        ];
      };
    in
    {
      devShells = eachSystem (
        system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          common = mkCommonConfig pkgs;
        in
        {
          default = self.devShells.${system}.snarkos-dev;
          snarkos-dev = pkgs.mkShell (
            common
            // {

              shellHook = ''
                echo "snarkOS development environment"
                echo "Rust version: $(rustc --version)"
                echo "Cargo version: $(cargo --version)"
              '';
            }
          );
        }
      );

      packages = eachSystem (
        system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          common = mkCommonConfig pkgs;

          # Read version from Cargo.toml
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in
        {
          default = self.packages.${system}.snarkos;
          snarkos-testnet = self.packages.${system}.snarkos.overrideAttrs (old: {
            buildFeatures = [ "test_network" ];
          });
          snarkos = pkgs.rustPlatform.buildRustPackage (
            common
            // {
              pname = cargoToml.package.name;
              version = cargoToml.package.version;

              # cleanup source tree
              src = nixpkgs.lib.cleanSourceWith {
                src = ./.;
                filter =
                  path: type:
                  # Exclude common build/cache directories
                  (!nixpkgs.lib.hasInfix "/target" path)
                  && (!nixpkgs.lib.hasInfix "/.git" path)
                  && (
                    # Include directories
                    (type == "directory")
                    ||
                      # Include .resources folder
                      (nixpkgs.lib.hasInfix "/.resources" path)
                    ||
                      # Include specific files
                      (nixpkgs.lib.hasSuffix ".lock" path)
                    || (nixpkgs.lib.hasSuffix ".rs" path)
                    || (nixpkgs.lib.hasSuffix ".toml" path)
                  );
              };
              cargoLock = {
                lockFile = ./Cargo.lock;
                outputHashes = {
                  "snarkvm-4.4.0" = "sha256-5NbGhCunhjxQA334wuM7WUqytIY1bP6OALIDwqVNuH8=";
                };
              };
            }
          );
        }
      );
    };
}
