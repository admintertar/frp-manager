# FRP Manager

Languages: [English](#english) | [简体中文](#简体中文)

## English

FRP Manager is a desktop client for managing local `frpc` profiles. It is built with Tauri 2, React, TypeScript, and Rust, and focuses on day-to-day profile operation rather than manual TOML editing.

### Features

- Manage multiple frpc profiles from one desktop app.
- Create and edit profiles with `serverAddr`, `serverPort`, token auth, and OIDC auth.
- Add, edit, delete, enable, and disable HTTP, TCP, and UDP proxies.
- Start, stop, and reload a running profile from the main window.
- Menu bar / tray controls for quick profile start and stop, proxy toggles, showing the window, and quitting the app.
- Managed frpc runtime: download the matching frpc package from `fatedier/frp` GitHub Releases instead of bundling a fixed binary.
- Runtime Settings show installed version, latest version, selected asset, local runtime path, and update status.
- Colored frpc logs: ANSI color output from frpc is rendered safely in the log panel.
- Local-first storage. Profiles and runtime metadata are stored in the application data directory.

### Runtime Model

FRP Manager does not ship with a bundled `frpc` binary. On first use, open Runtime Settings and download the runtime for the current platform. The app stores the runtime in its application data directory and records metadata in `runtime/current.json`.

If the runtime file is removed manually, FRP Manager detects it when the main window or Runtime Settings refreshes and marks the runtime as not installed.

### Development

Requirements:

- Node.js
- pnpm
- Rust
- Tauri platform dependencies for your OS

Install dependencies:

```bash
pnpm install
```

Run in development mode:

```bash
pnpm tauri dev
```

Build a release package:

```bash
pnpm tauri build
```

Frontend-only build:

```bash
pnpm build
```

### Tests

Run TypeScript checks:

```bash
pnpm typecheck
```

Run Rust tests:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Run focused UI/source tests:

```bash
node test/ansi-log.test.mjs
node test/port-input.test.mjs
pnpm test:runtime-management
pnpm test:proxy-types
pnpm test:app-name
```

### GitHub Release Packaging

The repository includes a GitHub Actions release workflow at `.github/workflows/release.yml`.

It can be triggered manually from the Actions tab or by pushing a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow first runs TypeScript checks, frontend build, source tests, and Rust tests. It then builds draft release packages for:

- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Windows x64
- Linux x64 on Ubuntu 22.04

The macOS build keeps the current app icon and uses ad-hoc signing in CI when no Apple signing certificate is configured. For public distribution, add Apple Developer ID signing and notarization secrets later. Linux packaging installs WebKitGTK 4.1 and appindicator development libraries so the Tauri window and tray/menu integration can compile.

Windows packages are unsigned unless a Windows code signing certificate is added to CI. Unsigned installers and newly released binaries can trigger SmartScreen or antivirus warnings because they do not have publisher reputation yet. FRP Manager app updates prefer the MSI asset on Windows, but public distribution should use an OV/EV code signing certificate with timestamped Windows bundle signatures.

Release publishing requires GitHub Actions to write repository contents. In GitHub, check Settings -> Actions -> General -> Workflow permissions and allow read and write permissions. If the repository or organization keeps `GITHUB_TOKEN` restricted, create a fine-grained personal access token with repository Contents read/write access and save it as the `RELEASE_TOKEN` secret.

The app package still does not include `frpc`; users install or update the matching runtime from Runtime Settings after launching FRP Manager.

### Data Storage

On macOS, app data is stored under:

```text
~/Library/Application Support/com.local.frpmanager
```

The app stores:

- `profiles/<profile-id>/profile.toml`
- `profiles/<profile-id>/logs/current.log`
- `runtime/current.json`
- downloaded frpc runtime files

### Notes

- Runtime downloads use GitHub Releases from `fatedier/frp`; GitHub API rate limits may affect update checks.
- Some antivirus products classify tunneling or proxy tools such as `frpc.exe` as potentially risky. FRP Manager downloads frpc from `fatedier/frp` Releases and verifies the official sha256 checksum before installing it.
- Quitting FRP Manager from the tray or Dock shuts down managed frpc processes.
- Proxy toggles rewrite the local profile TOML and reload the running profile when needed.

## 简体中文

FRP Manager 是一个用于管理本地 `frpc` 的桌面客户端。项目基于 Tauri 2、React、TypeScript 和 Rust 构建，目标是让日常启动、停止、编辑 profile 和 proxy 的操作尽量少依赖手动改 TOML。

### 功能特性

- 在一个桌面应用里管理多个 frpc Profile。
- 通过表单创建和编辑 Profile，支持 `serverAddr`、`serverPort`、token 认证和 OIDC 认证。
- 添加、编辑、删除、启用、停用 HTTP、TCP、UDP Proxy。
- 在主窗口里启动、停止、重载当前 Profile。
- 菜单栏 / 托盘支持快速启动停止 Profile、切换 Proxy、显示窗口和退出应用。
- 托管 frpc Runtime：不内置固定的 frpc 二进制文件，而是从 `fatedier/frp` GitHub Releases 下载当前平台对应的包。
- Runtime 设置界面显示已安装版本、最新版本、当前匹配的资产文件、本地路径和更新状态。
- 彩色日志显示：frpc 原生 ANSI 彩色输出会在日志面板中安全渲染。
- 本地优先存储。Profile、日志和 Runtime 元数据都保存在本机应用数据目录。

### Runtime 管理方式

FRP Manager 不会把 `frpc` 固定打进安装包。首次使用时，需要打开 Runtime Settings 下载当前系统对应的 frpc runtime。应用会把 runtime 保存到自己的应用数据目录，并用 `runtime/current.json` 记录版本、平台和路径。

如果你手动删除了本地 frpc 文件，FRP Manager 会在主窗口或 Runtime Settings 刷新时检测到，并把 runtime 状态标记为未安装。

### 开发

环境要求：

- Node.js
- pnpm
- Rust
- 当前系统对应的 Tauri 开发依赖

安装依赖：

```bash
pnpm install
```

启动开发模式：

```bash
pnpm tauri dev
```

打包发布版本：

```bash
pnpm tauri build
```

只构建前端：

```bash
pnpm build
```

### 测试

TypeScript 检查：

```bash
pnpm typecheck
```

Rust 测试：

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

常用的前端和源码回归测试：

```bash
node test/ansi-log.test.mjs
node test/port-input.test.mjs
pnpm test:runtime-management
pnpm test:proxy-types
pnpm test:app-name
```

### GitHub 发布打包

仓库里已经包含 GitHub Actions 发布 workflow：`.github/workflows/release.yml`。

可以在 Actions 页面手动触发，也可以推送版本 tag 触发：

```bash
git tag v0.1.0
git push origin v0.1.0
```

workflow 会先执行 TypeScript 检查、前端构建、源码测试和 Rust 测试，然后生成 draft release 包，覆盖：

- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Windows x64
- Linux x64，基于 Ubuntu 22.04

macOS 包会继续使用现在的应用图标；CI 里没有 Apple 证书时会使用 ad-hoc signing。后面如果要正式公开分发，再补 Apple Developer ID 签名和 notarization secrets。Linux 打包会安装 WebKitGTK 4.1 和 appindicator 开发库，保证 Tauri 窗口和托盘/菜单功能可以编译。

Windows 包在 CI 里没有 Windows 代码签名证书时会是未签名状态。未签名安装器和刚发布的新二进制可能触发 SmartScreen 或杀毒软件提醒，因为还没有发布者信誉。FRP Manager 在 Windows 自更新时会优先下载 MSI 资产；如果要公开稳定分发，应使用 OV/EV 代码签名证书给 Windows 包签名并加时间戳。

发布 Release 需要 GitHub Actions 拥有写入仓库内容的权限。需要在 GitHub 的 Settings -> Actions -> General -> Workflow permissions 里允许 read and write permissions。如果仓库或组织仍然限制 `GITHUB_TOKEN`，可以创建一个 fine-grained personal access token，给当前仓库 Contents read/write 权限，并保存为 `RELEASE_TOKEN` secret。

应用安装包仍然不会内置 `frpc`；用户启动 FRP Manager 后，在 Runtime Settings 里下载或升级当前系统对应的 frpc runtime。

### 数据存储

macOS 下应用数据默认位于：

```text
~/Library/Application Support/com.local.frpmanager
```

主要保存内容包括：

- `profiles/<profile-id>/profile.toml`
- `profiles/<profile-id>/logs/current.log`
- `runtime/current.json`
- 下载后的 frpc runtime 文件

### 说明

- Runtime 下载和更新检查依赖 `fatedier/frp` GitHub Releases，可能会受到 GitHub API 访问限制影响。
- 部分杀毒软件会把 `frpc.exe` 这类隧道/代理工具归类为潜在风险。FRP Manager 会从 `fatedier/frp` Releases 下载 frpc，并在安装前校验官方 sha256 checksum。
- 从托盘或 Dock 退出 FRP Manager 时，会退出由应用托管的 frpc 进程。
- 启用或停用 Proxy 会改写本地 profile TOML；如果对应 Profile 正在运行，会按需 reload。
