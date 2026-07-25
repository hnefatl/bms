{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    naersk.url = "github:nix-community/naersk";
    # Just used for rustChannelOf
    nixpkgs-mozilla.url = "github:mozilla/nixpkgs-mozilla";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      naersk,
      nixpkgs-mozilla,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import nixpkgs-mozilla)
        ];
        config = { };
      };

      toolchain =
        (pkgs.rustChannelOf {
          rustToolchain = ./rust-toolchain.toml;
          sha256 = "sha256-A1abGIbOtcBSdrUMhDGrER3pRM1hQP4fp9gh3Y4PKc8=";
        }).rust;

      naersk' = pkgs.callPackage naersk {
        cargo = toolchain;
        rustc = toolchain;
      };
    in
    rec {
      # The binary built from this flake.
      packages.${system}.default = naersk'.buildPackage rec {
        src = ./.;
        nativeBuildInputs = with pkgs; [
        ];
        RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
      };

      # The devshell to use when developing in this flake.
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs =
          with pkgs;
          [
            cargo
            pkg-config
          ]
          ++ packages.${system}.default.nativeBuildInputs;

        RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [];
      };
    };
}

