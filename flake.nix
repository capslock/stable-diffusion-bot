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
        overridableCrate = pkgs.lib.makeOverridable ({toolchain}: let
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        in
          craneLib.buildPackage {
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
          }) {toolchain = rust;};
        container = crate: {
          name = crate.pname;
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
          default = overridableCrate;
          container = pkgs.lib.makeOverridable ({crate}: pkgs.dockerTools.buildLayeredImage (container crate)) {crate = overridableCrate;};
          streamedContainer = pkgs.lib.makeOverridable ({crate}: pkgs.dockerTools.streamLayeredImage (container crate)) {crate = overridableCrate;};
        };
        checks = {inherit overridableCrate;};
        apps.default = flake-utils.lib.mkApp {
          drv = overridableCrate;
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
              package = mkOption {
                type = types.package;
                default = self.packages.${pkgs.system}.default;
                defaultText = literalExpression "self.packages.$\{pkgs.system\}.default";
                description = ''
                  stable-diffusion-bot package to use.
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
                pkg = cfg.package;
                configFile = settingsFormat.generate "sdbot-config.toml" cfg.settings;
                configs = [configFile] ++ lib.optional (cfg.telegramApiKeyFile != null) "$\{CREDENTIALS_DIRECTORY\}/sdbot.toml";
                args = lib.strings.concatMapStringsSep " " (file: "-c " + file) configs;
              in {
                Type = "exec";
                DynamicUser = true;
                ExecStart = "${pkg}/bin/stable-diffusion-bot --log-to-systemd ${args}";
                EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;
                Environment = cfg.environment;
                LoadCredential = mkIf (cfg.telegramApiKeyFile != null) "sdbot.toml:${cfg.telegramApiKeyFile}";
                NoNewPrivileges = true;
                PrivateTmp = true;
                PrivateDevices = true;
                DevicePolicy = "closed";
                ProtectSystem = "strict";
                ProtectHome = "read-only";
                ProtectControlGroups = true;
                ProtectKernelLogs = true;
                ProtectKernelModules = true;
                ProtectKernelTunables = true;
                RestrictNamespaces = true;
                RestrictAddressFamilies = ["AF_INET" "AF_INET6" "AF_UNIX"];
                RestrictRealtime = true;
                RestrictSUIDSGID = true;
                LockPersonality = true;
                CapabilityBoundingSet = [""];
                ProcSubset = "pid";
                ProtectClock = true;
                ProtectProc = "noaccess";
                ProtectHostname = true;
                SystemCallArchitectures = "native";
                SystemCallFilter = ["@system-service" "~@resources" "~@privileged"];
                UMask = "0077";
                PrivateUsers = true;
                MemoryDenyWriteExecute = true;
              };
            };
          };
        };
    };
}
