let
  sources = import ./npins;
  pkgsPath = sources.nixpkgs;

  sys = import "${pkgsPath}/nixos" {
    system = "x86_64-linux";
    configuration = { config, pkgs, lib, ... }: {
      imports = [
        "${pkgsPath}/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix"
        "${pkgsPath}/nixos/modules/installer/cd-dvd/channel.nix"
      ];
    };
  };
in
  sys.config.system.build.isoImage
