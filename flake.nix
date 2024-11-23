# SPDX-FileCopyrightText: 2021 Serokell <https://serokell.io/>
#
# SPDX-License-Identifier: CC0-1.0
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    crate2nix,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      inherit
        (import "${crate2nix}/tools.nix" {inherit pkgs;})
        generatedCargoNix
        ;

      web-serverBuildInputs = [pkgs.libgit2 pkgs.openssl];

      project =
        import (generatedCargoNix {
          name = "code-forge";
          src = ./.;
        }) {
          inherit pkgs;
          defaultCrateOverrides =
            pkgs.defaultCrateOverrides
            // {
              git2 = attrs: {
                nativeBuildInputs = [pkgs.pkg-config];
                buildInputs = [pkgs.libgit2];
              };
            };
        };
    in {
      packages.web-server = project.workspaceMembers.web-server.build;

      defaultPackage = self.packages.${system}.web-server;

      devShell = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.packages.${system};
        nativeBuildInputs = [pkgs.pkg-config];
        buildInputs =
          [
            pkgs.cargo
            pkgs.rust-analyzer
            pkgs.clippy
            pkgs.rustfmt
            pkgs.nodejs
            pkgs.nodePackages_latest.typescript-language-server
            pkgs.nodePackages_latest.svelte-language-server
            pkgs.nodePackages_latest.svelte-check
            pkgs.just
          ]
          ++ web-serverBuildInputs;
      };
    });
}
