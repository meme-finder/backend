{
  description = "MemeFinder Backend";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };

    import-cargo.url = "github:edolstra/import-cargo";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, import-cargo, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.stable.latest.default;
        inherit (import-cargo.builders) importCargo;

        nativeBuildInputs = with pkgs; [
          pkg-config
        ] ++ [
          rustVersion
        ];

        buildInputs = with pkgs; [
          openssl
        ];

        memefinder-backend = pkgs.stdenv.mkDerivation {
          name = "memefinder-backend";
          src = self;

          inherit buildInputs;

          nativeBuildInputs = [
            (importCargo { lockFile = ./Cargo.lock; inherit pkgs; }).cargoHome
          ] ++ nativeBuildInputs;

          buildPhase = ''
            cargo build --release --offline
          '';

          installPhase = ''
            install -Dm775 ./target/release/backend $out/bin/memefinder-backend
          '';
        };
      in
      {
        packages = {
          default = memefinder-backend;
          memefinder-backend = memefinder-backend;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = nativeBuildInputs ++ buildInputs;
        };
      }
    );
}
