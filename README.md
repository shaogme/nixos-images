# Custom NixOS Installer

This project builds a customized NixOS Minimal installer image (ISO), designed to provide a more flexible Base environment.

[中文文档 (Chinese Version)](README_CN.md)

## Key Differences from Official Images

### 1. Software Source Strategy (Unstable Channel)
- **Nixpkgs Unstable**: This uses the `nixpkgs` version locked in the current repository, rather than the default Stable branch of official ISOs. This means you can install the latest packages out of the box.
- **Offline Support**: A complete Nixpkgs source tree is pre-bundled in the image via `channel.nix`, allowing basic package management operations in offline environments.

### 2. User Account System (Root Only)
- **Root Only**: The default `nixos` user found in official images has been completely removed.
- **Auto-login**: The system automatically logs in as `root` to the console upon boot.
- **Default Password**: The default password for `root` is set to **`root`**.
  - *Note*: If a password is passed via the `live.nixos.passwd` kernel parameter, it will modify the root password.

### 3. Modular Configuration
- Instead of relying on the official `installation-cd-minimal.nix` as a black box, core configurations are decoupled into the `nix/iso/` directory:
  - `nix/iso/installer.nix`: Core installation environment configuration (SSH, hardware support, etc.).
  - `nix/iso/channel.nix`: Software source bundling logic.
  - `nix/iso/minimal.nix`: System trimming strategy.

## Build Instructions

Ensure Nix is installed (Flakes support is recommended but `nix-build` works too):

```bash
nix-build iso.nix
```

After the build completes, the `result` symlink in the current directory points to the generated ISO image file.

## Automated Build & Release Strategy

This project configures a daily automated build process via GitHub Actions, ensuring the image always follows upstream `nixos-unstable` updates.

### Automation Workflow (`.github/workflows/iso-release.yml`)
- **Trigger Time**: Daily at 03:00 Beijing Time (19:00 UTC).
- **Build Process**:
  1. Automatically update `nixpkgs` sources locked by `npins`.
  2. Build the new ISO image.
  3. Publish the new image via the Release Manager tool.
  4. Commit and push the updated `npins/sources.json` and `releases.json`.

### Version Retention Strategy (`tools/release-manager`)
To prevent an infinite growth of Releases and maintain a reasonable update density, we implement a real-time rolling release strategy:

1. **Minimum Interval (7 Days)**: Versions are only retained if the time interval between existing versions exceeds 7 days; otherwise, older intermediate versions are cleaned up. This ensures the release list isn't flooded with daily builds that have minor differences.
2. **Maximum Count (7 Versions)**: The system retains at most the latest 7 Release versions. When this limit is exceeded, the oldest version is automatically removed.

Through this strategy, we ensure users can always access the latest build while maintaining a clean version list with historical gradients.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
