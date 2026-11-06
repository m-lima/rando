{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    helper.url = "github:m-lima/nix-template";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      helper,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        outputs =
          (helper.lib.rust.helper inputs system ./. {
            formatters = {
              prettier.enable = true;
            };
          }).outputs;
        webRoot = pkgs.runCommand "build-web-root" { } ''
          mkdir $out
          cp ${./index.html} $out/index.html
        '';
      in
      outputs
      // {
        packages = {
          inherit webRoot;
          server = outputs.packages.default;
        };
      }
    );
}
