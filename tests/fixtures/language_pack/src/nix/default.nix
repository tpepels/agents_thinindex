{ pkgs ? import <nixpkgs> {} }:

let
  importedModule = import ./module.nix;

  mkNixWidget =
    name:
    {
      inherit name;
      kind = "class NixStringFake";
    };

  nixPackage = {
    pname = "thinindex-nix-widget";
  };
in
{
  inherit importedModule mkNixWidget nixPackage;
}
