{ pkgs ? import <nixpkgs> { system = "aarch64-darwin"; } }:
pkgs.dockerTools.buildLayeredImage {
  name = "builder";
  tag = "latest";
  contents = [ 
  (pkgs.rustPlatform.buildRustPackage {
      cargoSha256 = "";
      name = "builder";
      src = ../services/builder;
    })
  ];
}
