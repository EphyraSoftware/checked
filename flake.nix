{
  description = "Flake for Holochain app development";

  inputs = {
    versions.url  = "github:holochain/holochain?dir=versions/weekly";

    holochain-flake.url = "github:holochain/holochain";
    holochain-flake.inputs.versions.follows = "versions";

    nixpkgs.follows = "holochain-flake/nixpkgs";
    flake-parts.follows = "holochain-flake/flake-parts";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake
      {
        inherit inputs;
      }
      {
        systems = builtins.attrNames inputs.holochain-flake.devShells;
        perSystem =
          { inputs'
          , config
          , pkgs
          , system
          , ...
          }: {
            devShells.default = pkgs.mkShell {
              inputsFrom = [ inputs'.holochain-flake.devShells.holonix ];
              packages = [
                pkgs.nodejs_20
                pkgs.gnupg
                pkgs.pinentry
              ];

              shellHook = ''
                export GNUPGHOME=$(pwd)/.gnupg

                if [[ ! -d $GNUPGHOME ]]; then
                  gpg --list-keys --no-keyring 2>&1 > /dev/null
                  rm $GNUPGHOME/common.conf
                  echo "pinentry-program $(which pinentry)" > $GNUPGHOME/gpg-agent.conf
                  pkill gpg-agent
                  gpg-agent --daemon
                fi
              '';
            };
          };
      };
}
