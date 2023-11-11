{
  description = "Telegram bot for Stable Diffusion, written in Rust.";

  # Flake inputs
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs"; # also valid: "nixpkgs"
    rust-overlay.url = "github:oxalica/rust-overlay"; # A helper for Rust + Nix
  };

  # Flake outputs
  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    # Overlays enable you to customize the Nixpkgs attribute set
    overlays = [
      # Makes a `rust-bin` attribute available in Nixpkgs
      (import rust-overlay)
      # Provides a `rustToolchain` attribute for Nixpkgs that we can use to
      # create a Rust environment
      (self: super: {
        rustToolchain = super.rust-bin.stable.latest.default.override {
          extensions = ["rust-src"];
        };
      })
    ];

    # Systems supported
    allSystems = [
      "x86_64-linux" # 64-bit Intel/AMD Linux
      "aarch64-linux" # 64-bit ARM Linux
      "x86_64-darwin" # 64-bit Intel macOS
      "aarch64-darwin" # 64-bit ARM macOS
    ];

    # Mac frameworks needed for build & development
    macFrameworks = pkgs: let
      frameworks = pkgs.darwin.apple_sdk.frameworks;
    in
      with frameworks; [
        CoreFoundation
        CoreServices
        Security
        SystemConfiguration
      ];

    # Helper to provide system-specific attributes
    forAllSystems = f:
      nixpkgs.lib.genAttrs allSystems (system:
        f {
          pkgs = import nixpkgs {inherit overlays system;};
        });
  in {
    packages = forAllSystems ({pkgs}: {
      default = pkgs.rustPlatform.buildRustPackage {
        name = "stable-diffusion-bot";
        version = "0.1.0";
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
        nativeBuildInputs = [
          pkgs.pkg-config
        ];
        buildInputs =
          [
            pkgs.openssl
            pkgs.sqlite
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (macFrameworks pkgs);
      };
      container = let
        package = self.packages.${pkgs.system}.default;
      in
        pkgs.dockerTools.buildImage {
          name = package.name;
          tag = package.version;
          created = "now";
          config = {
            Cmd = ["${package}/bin/${package.name}"];
          };
        };
    });
    # Development environment output
    devShells = forAllSystems ({pkgs}: {
      default = pkgs.mkShell {
        buildInputs = [pkgs.pkg-config pkgs.openssl.dev pkgs.sqlite];
        # The Nix packages provided in the environment
        packages =
          (with pkgs; [
            # The package provided by our custom overlay. Includes cargo, Clippy, cargo-fmt,
            # rustdoc, rustfmt, and other tools.
            rustToolchain
          ])
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (
            with pkgs;
              [
                libiconv
              ]
              ++ macFrameworks pkgs
          );
      };
    });
    nixosModules.default = {
      config,
      lib,
      pkgs,
      ...
    }: let
      cfg = config.services.stableDiffusionBot;
      settingsFormat = pkgs.formats.toml {};
    in
      with lib; {
        options = {
          services.stableDiffusionBot = {
            enable = mkOption {
              default = false;
              type = with types; bool;
              description = ''
                Start the stable diffusion bot.
              '';
            };
            environmentFile = mkOption {
              example = "./sdbot.env";
              type = nullOr types.str;
              description = lib.mdDoc ''
                File which contains environment settings for the stable-diffusion-bot service.
              '';
            };
            settings = lib.mkOption {
              type = settingsFormat.type;
              default = {};
              description = ''
                Configuration for stable-diffusion-bot, see
                <link xlink:href="https://www.github.com/capslock/stable-diffusion-bot"/>
                for supported settings.
              '';
            };
          };
        };

        config = mkIf cfg.enable {
          systemd.services.stableDiffusionBot = {
            wantedBy = ["multi-user.target"];
            after = ["network-online.target"];
            description = "Stable Diffusion Bot";
            serviceConfig = let
              pkg = self.packages.${pkgs.system}.default;
              configFile = settingsFormat.generate "sdbot-config.toml" cfg.settings;
            in {
              ExecStart = "${pkg}/bin/stable-diffusion-bot -c ${configFile}";
              EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;
            };
          };
        };
      };
  };
}
