{
  description = "Flake for Holochain app development";

  inputs = {
    versions.url = "github:holochain/holochain?dir=versions/weekly";

    holochain-flake.url = "github:holochain/holochain";
    holochain-flake.inputs.versions.follows = "versions";

    nixpkgs.follows = "holochain-flake/nixpkgs";
    flake-parts.follows = "holochain-flake/flake-parts";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
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
      systems = builtins.attrNames inputs.holochain-flake.devShells;
      perSystem =
        { inputs'
        , config
        , pkgs
        , system
        , lib
        , ...
        }:
        let
          opensslStatic =
            if system == "x86_64-darwin"
            then pkgs.openssl
            else pkgs.pkgsStatic.openssl;

          craneLib = crane.lib.${system};
          src = craneLib.cleanCargoSource (craneLib.path ./checked_cli);

          checkedCliCrateInfo = craneLib.crateNameFromCargoToml { cargoToml = ./checked_cli/Cargo.toml; };

          # Common arguments can be set here to avoid repeating them later
          commonArgs = {
            pname = checkedCliCrateInfo.pname;
            version = checkedCliCrateInfo.version;

            inherit src;
            strictDeps = true;

            buildInputs = with pkgs; [
              # Some Holochain crates link against openssl
              openssl
              opensslStatic
            ];

            nativeBuildInputs = with pkgs; [
              # To build openssl-sys
              perl
              pkg-config
            ];
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
            inputsFrom = [ inputs'.holochain-flake.devShells.holonix ];

            packages = with pkgs; [
              nodejs_20
              minisign
              libsodium
              lcov
            ];

            shellHook = ''
              # This is enough to get libsodium-sys-stable to link against the libsodium we're providing
              export SODIUM_LIB_DIR="${pkgs.libsodium}/lib/"
              export SODIUM_SHARED="1"

              # Irritatingly, the above isn't enough. Because `lair_keystore` depends on `lair_keystore_api` in its `build.rs`
              # which apparently doesn't go through the same build process. That's probably enough justification that it should
              # not have that dependency but for now, we get around the problem by configuring the linker directly, outside Cargo's
              # build process...
              export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$SODIUM_LIB_DIR
            '';
          };
        };
    };
}
