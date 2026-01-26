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

      # --- 深度精简配置 ---

      # 重写系统软件包列表
      # 使用 mkForce 覆盖默认引入的大量工具 (来自 profiles/base.nix)
      environment.systemPackages = lib.mkForce (with pkgs; [
        # 编辑器
        neovim
        nano
        
        # 核心工具
        git
        curl
        wget
        rsync
        htop
        ripgrep
        fd
        tree
        which
        file
        
        # 磁盘与文件系统工具
        parted
        gptfdisk # gdisk
        dosfstools # mkfs.vfat etc
        btrfs-progs # btrfs tools
        e2fsprogs # mkfs.ext4 etc
        
        # 压缩解压
        unzip
        zip
        gzip
        
        # 网络工具
        iproute2
        iputils # ping
        dnsutils # dig, nslookup
      ]);

      # 禁用不需要的文档
      documentation.enable = lib.mkForce false;
      documentation.doc.enable = lib.mkForce false;
      documentation.man.enable = lib.mkForce false;
      documentation.nixos.enable = lib.mkForce false;

      # 其他优化
      programs.git.enable = true; # 确保 git 配置生效
      boot.loader.grub.memtest86.enable = lib.mkForce false; # 使用 mkForce 强制禁用 memtest86
      
      # 压缩设置
      isoImage.squashfsCompression = "zstd -Xcompression-level 19";
    };
  };
in
  sys.config.system.build.isoImage
