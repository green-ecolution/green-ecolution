{
  description = "Development shell for green ecolution frontend";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  } @ inputs: (flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            eslint = {
              language = "javascript";
              args = ["--fix" "--ext" ".ts,.tsx"];
              files = ["src/**/*.{ts,tsx}"];
            };
            prettier = {
              args = ["--write"];
              files = ["src/**/*.{js,jsx,ts,tsx,json,css,scss,md}"];
            };
            typescript = {
              args = ["--noEmit" "--strict"];
              files = ["src/**/*.{ts,tsx}"];
            };
            test = {
              args = ["--bail" "--findRelatedTests"];
              files = ["src/**/*.{test.js,test.ts,test.jsx,test.tsx}"];
            };
          };
        };
      in {
        # devShells."x86_64-linux".default = import ./shell.nix { inherit pkgs; };
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            nodejs_24
            pnpm
            eslint
            prettierd
            docker
          ];

          shellHook = ''
            ${pre-commit-check.shellHook}
          '';
        };
      }
    ));
}
