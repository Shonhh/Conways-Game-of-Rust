{
  description = "Conway's Game of Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;

          # The source is the current directory
          src = ./.;

          # Nix needs the Cargo.lock to fetch dependencies deterministically
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Runtime dependencies (none strictly needed for pure Crossterm/Ratatui,
          # but useful to know where to put them if you add audio/graphics later)
          buildInputs = [ ];

          # Build-time tools
          nativeBuildInputs = [ ];

          meta = with pkgs.lib; {
            description = "A generic implementation of Conway's Game of Life in Rust";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        # Optional: A development shell with tools pre-installed
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
          packages = with pkgs; [
            rust-analyzer
            clippy
            rustfmt
          ];
        };
      }
    );
}
