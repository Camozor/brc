{
  description = "1 billion row challenge";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable"; };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      defaultPackage.${system} = pkgs.mkShell {
        buildInputs = with pkgs; [
          nodejs_24

          rustfmt
          rustc
          clippy
          pkg-config
          cmake
          just
        ];
      };

    };
}
