{ config, pkgs, lib, options, modulesPath, ... }:

let
  mkImageMediaOverride = lib.mkOverride 60;
in
{
  imports = [
    ./channel.nix
    "${modulesPath}/installer/scan/detected.nix"
    "${modulesPath}/installer/scan/not-detected.nix"
  ];

  # --- From installation-cd-base.nix ---

  hardware.enableAllHardware = true;

  console.packages = options.console.packages.default ++ [ pkgs.terminus_font ];

  isoImage.makeEfiBootable = true;
  isoImage.makeUsbBootable = true;
  
  # Optimization: Disable Legacy/BIOS boot support to reduce size and complexity
  # We assume modern hardware (UEFI)
  isoImage.makeBiosBootable = false;
  
  # Optimization: Remove heavy Grub theme
  isoImage.grubTheme = lib.mkForce null;

  # Optimization: Faster boot timeout (default 10s)
  boot.loader.timeout = lib.mkForce 2;

  boot.loader.grub.memtest86.enable = true;

  # Filesystem configuration for the ISO
  swapDevices = mkImageMediaOverride [ ];
  fileSystems = mkImageMediaOverride config.lib.isoFileSystems;
  boot.initrd.luks.devices = mkImageMediaOverride { };

  boot.postBootCommands = ''
    for o in $(</proc/cmdline); do
      case "$o" in
        live.nixos.passwd=*)
          set -- $(IFS==; echo $o)
          echo "root:$2" | ${pkgs.shadow}/bin/chpasswd
          ;;
      esac
    done
    
    # Ensure /mnt exists for key mounting
    mkdir -p /mnt
  '';

  environment.defaultPackages = with pkgs; [
    rsync
  ];

  programs.git.enable = lib.mkDefault true;
  
  system.stateVersion = lib.mkDefault lib.trivial.release;

  # --- From installation-device.nix ---

  system.nixos.variant_id = lib.mkDefault "installer";

  users.users.root.initialHashedPassword = "$6$JYLMHGrBPn1P8OW2$G/JC0qiHvsLLqR4ORvt1n78cXe8DChQQu6j8HQ24xLLGHPNaDq7FX7qjb4coeq5VTw1Ol8fXT84Dl4cwOI.AC/";

  security.polkit.enable = true;

  services.getty.autologinUser = "root";
  services.getty.helpLine = ''
    The "root" account password is "root".
    
    To log in over ssh set a different password with `passwd` if desired.
    To set up wifi run `nmtui`.
  '';

  services.openssh = {
    enable = lib.mkDefault true;
    settings.PermitRootLogin = lib.mkDefault "yes";
  };

  networking.networkmanager.enable = true;
  networking.firewall.logRefusedConnections = lib.mkDefault false;

  environment.variables.GC_INITIAL_HEAP_SIZE = "1M";
  boot.kernel.sysctl."vm.overcommit_memory" = "1";

  system.extraDependencies = with pkgs; [
    stdenvNoCC
    busybox
    makeInitrdNGTool
  ] ++ jq.all;

  boot.swraid.enable = true;
  boot.swraid.mdadmConf = "PROGRAM ${pkgs.coreutils}/bin/true";
  
  environment.etc."systemd/pstore.conf".text = ''
    [PStore]
    Unlink=no
  '';
}
