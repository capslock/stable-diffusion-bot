{
  description = "Telegram bot for Stable Diffusion, written in Rust.";

  # Flake inputs
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs"; # also valid: "nixpkgs"
    rust-overlay = {
      url = "github:oxalica/rust-overlay"; # A helper for Rust + Nix
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  # Flake outputs
  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
    flake-utils,
  }: let
    overlays = [
      (import rust-overlay)
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
  in
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        rust = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rust;
        crate = craneLib.buildPackage {
          pname = "stable-diffusion-bot";
          version = "0.1.0";
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;
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
        container = {
          name = crate.name;
          tag = "latest";
          created = "now";
          contents = [pkgs.cacert];
          config = {
            Labels = {
              "org.opencontainers.image.source" = "https://github.com/capslock/stable-diffusion-bot";
              "org.opencontainers.image.description" = "Stable Diffusion Bot";
              "org.opencontainers.image.licenses" = "MIT";
            };
            Entrypoint = ["${crate}/bin/${crate.name}"];
          };
        };
      in {
        packages = {
          default = crate;
          container = pkgs.dockerTools.buildLayeredImage container;
          streamedContainer = pkgs.dockerTools.streamLayeredImage container;
        };
        checks = {inherit crate;};
        apps.default = flake-utils.lib.mkApp {
          drv = crate;
        };
        # Development environment output
        devShells.default = craneLib.devShell {
          buildInputs = [pkgs.pkg-config pkgs.openssl.dev pkgs.sqlite];
          checks = self.checks.${system};
          # The Nix packages provided in the environment
          packages = pkgs.lib.optionals pkgs.stdenv.isDarwin (
            with pkgs;
              [
                libiconv
              ]
              ++ macFrameworks pkgs
          );
        };
      }
    )
    // {
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
                type = with types; nullOr str;
                default = null;
                description = ''
                  File which contains environment settings for the stable-diffusion-bot service.
                '';
              };
              environment = mkOption {
                example = "RUSTLOG=info";
                type = with types; str;
                default = "\"RUSTLOG=info,hyper=error\"";
                description = ''
                  Environment settings for the stable-diffusion-bot service.
                '';
              };
              telegramApiKeyFile = mkOption {
                example = "./sdbot.toml";
                type = with types; nullOr str;
                default = null;
                description = ''
                  TOML file containing an `api_key` entry set to the telegram API key to use.
                  May also contain other configuration, see
                  <link xlink:href="https://www.github.com/capslock/stable-diffusion-bot"/>
                  for supported settings.
                '';
              };
              settings = mkOption {
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
                pkg = self.packages.${system}.default;
                configFile = settingsFormat.generate "sdbot-config.toml" cfg.settings;
                configs = [configFile] ++ lib.optional (cfg.telegramApiKeyFile != null) "$\{CREDENTIALS_DIRECTORY\}/sdbot.toml";
                args = lib.strings.concatMapStringsSep " " (file: "-c " + file) configs;
              in {
                Type = "exec";
                DynamicUser = true;
                ExecStart = "${pkg}/bin/stable-diffusion-bot ${args}";
                EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;
                Environment = cfg.environment;
                LoadCredential = mkIf (cfg.telegramApiKeyFile != null) "sdbot.toml:${cfg.telegramApiKeyFile}";
              };
            };
          };
        };
    };
}
