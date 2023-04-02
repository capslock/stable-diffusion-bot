{
  description = "Example Rust development environment for Zero to Nix";

  # Flake inputs
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs"; # also valid: "nixpkgs"
    rust-overlay.url = "github:oxalica/rust-overlay"; # A helper for Rust + Nix
  };

  # Flake outputs
  outputs = { self, nixpkgs, rust-overlay }:
    let
      # Overlays enable you to customize the Nixpkgs attribute set
      overlays = [
        # Makes a `rust-bin` attribute available in Nixpkgs
        (import rust-overlay)
        # Provides a `rustToolchain` attribute for Nixpkgs that we can use to
        # create a Rust environment
        (self: super: {
          rustToolchain = super.rust-bin.stable.latest.default;
        })
      ];

      # Systems supported
      allSystems = [
        "x86_64-linux" # 64-bit Intel/AMD Linux
        "aarch64-linux" # 64-bit ARM Linux
        "x86_64-darwin" # 64-bit Intel macOS
        "aarch64-darwin" # 64-bit ARM macOS
      ];

      # Helper to provide system-specific attributes
      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit overlays system; };
      });
    in
    {
      packages = forAllSystems ({ pkgs }: {
        default = pkgs.rustPlatform.buildRustPackage {
          name = "stable-diffusion-bot";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          buildInputs = [
            pkgs.openssl
            pkgs.sqlite
          ];

          SQLITE3_LIB_DIR = "${pkgs.sqlite.out}/lib";
          SQLITE3_INCLUDE_DIR = "${pkgs.sqlite.dev}/include";

          doCheck = false;
        };
      });
      # Development environment output
      devShells = forAllSystems ({ pkgs }: {
        default = pkgs.mkShell {
          buildInputs = [ pkgs.pkg-config pkgs.openssl.dev pkgs.sqlite ];
          # The Nix packages provided in the environment
          packages = (with pkgs; [
            # The package provided by our custom overlay. Includes cargo, Clippy, cargo-fmt,
            # rustdoc, rustfmt, and other tools.
            rustToolchain
          ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);
        };
      });
    };
}
