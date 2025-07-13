{
  description = "Todo Tool Flake";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustVersion = "2025-07-13";
        rust = (pkgs.rust-bin.nightly."${rustVersion}".default.override {
          extensions = [ "rust-src" ];
        });
        buildInputs = with pkgs; [];
        nativeBuildInputs = with pkgs; [
          rust
          pkg-config
          gcc
        ] ++ buildInputs;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust;
          rustc = rust;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs;
        };

        packages.default = rustPlatform.buildRustPackage rec {
          pname = "todo";
          version = "0.1.0";
          
          src = ./.;

          cargoHash = "sha256-zgmp0L1VbWbnqFUKrPbA+T8P5WceZQXtlWq+ikuc2Qw=";

          inherit buildInputs nativeBuildInputs;

          # postInstall = ''
          #   mkdir -p $out/share/applications
          #   cp $src/timekeeper.desktop $out/share/applications/timekeeper.desktop
          # '';
        };
      }
    );
}
