let
  sources = import ./npins;
  pkgsPath = sources.nixpkgs;

  sys = import "${pkgsPath}/nixos" {
    system = "x86_64-linux";
    configuration = { config, pkgs, lib, ... }: {
      imports = [
        # Cleaned up imports - using official modules where appropriate
        "${pkgsPath}/nixos/modules/installer/cd-dvd/iso-image.nix"
        # "${pkgsPath}/nixos/modules/profiles/base.nix"
        
        # Local modular configuration
        ./nix/iso/installer.nix
        ./nix/iso/base.nix
        ./nix/iso/minimal.nix
      ];

      # --- User Customizations ---

      # Ensure git is enabled
      programs.git.enable = true;

      # Disable Memtest86+ (Force override installer default)
      boot.loader.grub.memtest86.enable = lib.mkForce false;

      # Disable channel
      system.installer.channel.enable = false;
      
      # Compression settings
      isoImage.squashfsCompression = "zstd -Xcompression-level 19";
    };
  };
in
  sys.config.system.build.isoImage
