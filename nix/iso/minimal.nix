{ lib, pkgs, ... }:

let
  inherit (lib) mkDefault mkForce;
in
{
  # Documentation
  documentation = {
    enable = mkForce false;
    doc.enable = mkForce false;
    info.enable = mkForce false;
    man.enable = mkForce false;
    nixos.enable = mkForce false;
  };

  # Environment
  environment = {
    # Perl is a default package.
    defaultPackages = mkDefault [ ];
    stub-ld.enable = mkDefault false;
  };

  # Programs
  programs = {
    command-not-found.enable = mkDefault false;
    fish.generateCompletions = mkDefault false;
  };

  # Services
  services = {
    logrotate.enable = mkDefault false;
    udisks2.enable = mkDefault false;
  };

  # XDG
  xdg = {
    autostart.enable = mkDefault false;
    icons.enable = mkDefault false;
    mime.enable = mkDefault false;
    sounds.enable = mkDefault false;
  };
  
  # Fonts
    fonts.fontconfig.enable = mkDefault false;
}
