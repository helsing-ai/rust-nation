{
  description = "Flake for building all RustNation things!";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";


    nixos-hardware = { url = "github:NixOS/nixos-hardware/master"; };
    nixos-generators = { url = "github:nix-community/nixos-generators"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = { self, nixpkgs, nixos-generators, nixos-hardware, crane, flake-utils, ... }: let
    wifi = (builtins.fromTOML (builtins.readFile ./pi/wifi.toml)).credentials;

    teams' = builtins.fromTOML (builtins.readFile ./pi/teams.toml);
    teams = nixpkgs.lib.attrsets.mapAttrsToList
      (name: attrs: { inherit (attrs) drone key; inherit name wifi; })
      teams';

    configuration = team: nixpkgs.lib.nixosSystem {
      system = "aarch64-linux";

      modules = [
        nixos-hardware.nixosModules.raspberry-pi-4
        "${nixpkgs}/nixos/modules/profiles/minimal.nix"
        ./pi/configuration.nix
      ];

      specialArgs = { inherit team; };
    };

    nixosImage = config: (config.extendModules {
      modules = [
        "${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix"
        { disabledModules = [ "profiles/base.nix" ]; sdImage.compressImage = true; }
      ];
    }).config.system.build.sdImage;

    mergeAttributes = nixpkgs.lib.lists.foldl nixpkgs.lib.recursiveUpdate {};
  in mergeAttributes [
    {
      nixosConfigurations = builtins.listToAttrs (
        builtins.map
          (team: { name = team.name; value = configuration team; })
          teams
      );
    }

    {
      nixosImages = builtins.listToAttrs (
        builtins.map
          (team: { name = team.name; value = nixosImage self.nixosConfigurations.${team.name}; })
          teams
      );
    }

    {
      packages.aarch64-linux = self.nixosImages;
    }

    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        craneLib = crane.lib.${system};

        hackathon = craneLib.buildPackage {
          src = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = path: type: (builtins.match ".*/resources/[^/]+$" path != null) || (craneLib.filterCargoSources path type);
          };
          strictDeps = true;

          buildInputs = [
            pkgs.gtk3
            pkgs.gtk4
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Cocoa
            pkgs.libiconv
          ];

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.libcxx
          ];

          # work around for https://github.com/PyO3/pyo3/issues/1800
          NIX_LDFLAGS = if pkgs.stdenv.isDarwin then "-l${pkgs.libcxx.cxxabi.libName}" else "";
        };

        images = 
          builtins.listToAttrs (builtins.map (team: {
            name = team.name;
            value = pkgs.writeShellApplication {
              name = "build";
              runtimeInputs = with pkgs; [ coreutils zstd ];
              text = ''
                cat ${self.nixosImages.${team.name}}/sd-image/*.img.zst | zstd -d -c > ${team.name}.img
              '';
            };
          }) teams);
      in
      {
        checks = { inherit hackathon; };

        packages = { default = hackathon; } // images; 

        apps.default = flake-utils.lib.mkApp { drv = hackathon; };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = [];

          # work around for https://github.com/PyO3/pyo3/issues/1800
          NIX_LDFLAGS = if pkgs.stdenv.isDarwin then "-l${pkgs.libcxx.cxxabi.libName}" else "";
        };
      }))
    ];
}
