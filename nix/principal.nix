{ pkgs ? import <nixpkgs> { system = "x86_64-linux"; } }:
pkgs.dockerTools.buildLayeredImage {
  name = "principal";
  tag = "latest";
  contents = [ 
  (pkgs.rustPlatform.buildRustPackage {
      cargoSha256 = "";
      name = "principal";
      src = ../principal;
    })
  ];
}