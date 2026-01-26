# 自定义 NixOS 安装镜像 (Custom NixOS Installer)

本项目构建了一个定制化的 NixOS Minimal 安装镜像 (ISO)，旨在提供更灵活的 Base 环境。

## 与官方镜像的主要区别

### 1. 软件源策略 (Unstable Channel)
- **Nixpkgs Unstable**: 也就是当前仓库锁定的 nixpkgs 版本，而非官方 ISO 默认的 Stable 分支。这意味着您开箱即可安装最新的软件包。
- **离线支持**: 镜像内通过 `channel.nix` 预置了完整的 Nixpkgs 源码树，允许在无网络环境下进行基础的包管理操作。

### 2. 用户账户体系 (Root Only)
- **仅 Root 用户**: 彻底移除了官方镜像默认的 `nixos` 普通用户。
- **自动登录**: 系统启动后会自动以 `root` 身份登录到控制台。
- **默认密码**: `root` 用户的默认密码已设置为 **`root`**。
  - *注意*: 如果通过 `live.nixos.passwd` 内核参数传递密码，将会修改 root 的密码。

### 3. 配置模块化
- 不再黑盒依赖官方的 `installation-cd-minimal.nix`，而是将核心配置解耦到 `nix/iso/` 目录下：
  - `nix/iso/installer.nix`: 核心安装环境配置（SSH, 硬件支持等）。
  - `nix/iso/channel.nix`: 软件源捆绑逻辑。
  - `nix/iso/minimal.nix`: 系统精简策略。

## 构建方法

确保已安装 Nix 并启用 Flakes（或使用传统的 nix-build）：

```bash
nix-build iso.nix
```

构建完成后，当前目录下名为 `result` 的软链接即指向生成的 ISO 镜像文件。

## 自动构建与发布策略

本项目配置了 GitHub Actions 每日自动构建流程，确保镜像始终跟随上游 `nixos-unstable` 更新。

### 自动化工作流 (`.github/workflows/iso-release.yml`)
- **触发时间**: 每日北京时间凌晨 03:00 (UTC 19:00)。
- **构建流程**:
  1. 自动更新 `npins` 锁定的 Nixpkgs 源码。
  2. 构建新的 ISO 镜像。
  3. 通过 Release Manager 工具发布新镜像。
  4. 提交并推送更新后的 `npins/sources.json` 和 `releases.json`。

### 版本保留策略 (`tools/release-manager`)
为了防止 Release 数量无限增长并保持合理的更新密度，我们实施了即时的版本轮替策略 (Rolling Release Strategy)：

1. **最小间隔 (7天)**: 只有当现有版本之间的时间间隔超过 7 天时才会被保留，否则较旧的中间版本将被清理。这确保了发布列表中不会充斥着差异微小的每日构建。
2. **最大数量 (7个)**: 系统最多保留最新的 7 个 Release 版本。当超出限制时，最旧的版本将被自动移除。

通过这种策略，我们既保证了用户总能获取到最新的构建，又维护了一个干净、有历史梯度的版本列表。

## 许可证

本项目采用 MIT 许可证，详情请见 [LICENSE](LICENSE) 文件。