{
  description = "Flake for Holochain app development";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=24.05";

    holonix = {
        url = "github:holochain/holonix/main-0.3";

        inputs = {
            nixpkgs.follows = "nixpkgs";
            flake-parts.follows = "flake-parts";
            crane.follows = "crane";
        };
    };

    flake-parts.url = "github:hercules-ci/flake-parts";

    crane = {
      url = "github:ipetkov/crane";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    inputs@{ flake-parts
    , crane
    , fenix
    , advisory-db
    , ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = builtins.attrNames inputs.holonix.devShells;
      perSystem =
        { inputs'
        , config
        , pkgs
        , system
        , lib
        , ...
        }:
        let
          craneLib = crane.lib.${system};
          src = craneLib.cleanCargoSource (craneLib.path ./checked_cli);

          checkedCliCrateInfo = craneLib.crateNameFromCargoToml { cargoToml = ./checked_cli/Cargo.toml; };

          # Common arguments can be set here to avoid repeating them later
          commonArgs = {
            pname = checkedCliCrateInfo.pname;
            version = checkedCliCrateInfo.version;

            inherit src;
            strictDeps = true;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          checkedCli = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        in
        {
          formatter = pkgs.nixpkgs-fmt;

          checks = {
            inherit checkedCli;

            checkedCliClippy = craneLib.cargoClippy (commonArgs // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            });

            checkedCliDoc = craneLib.cargoDoc (commonArgs // {
              inherit cargoArtifacts;
            });

            checkedCliFmt = craneLib.cargoFmt (commonArgs // {
              inherit src;
            });

            # Multiple packages failing through Holochain dependencies...
            # checkedCliAudit = craneLib.cargoAudit {
            #    inherit src advisory-db;
            # };
          };

          packages = {
            default = checkedCli;
          };

          apps.default = checkedCli;

          devShells.default = pkgs.mkShell {
            packages = (with inputs'.holonix.packages; [
                holochain
                lair-keystore
                hc-launch
                hc-scaffold
                # hn-introspect
                rust
            ]) ++ (with pkgs; [
              nodejs_20
              minisign
              upx # For binary size optimisation. Not currently working with `checked_cli`, try again later
              binaryen # For wasm-opt, optimising wasms before packaging
            ]);
          };
        };
    };
}
