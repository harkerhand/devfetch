# devfetch

一个用 Rust 编写的开发环境探测工具，用于打印机器上的开发环境信息（如 Node、npm、pnpm、Rust、Git、数据库、浏览器等）。

## 快速开始

### 1) 脚本安装（推荐）

支持直接远程执行脚本。脚本内部会从 GitHub Release 下载对应平台二进制，不会本地编译。

Linux/macOS：

```bash
curl -fsSL https://raw.githubusercontent.com/harkerhand/devfetch/master/scripts/install.sh | bash
```

Windows PowerShell：

```powershell
irm https://raw.githubusercontent.com/harkerhand/devfetch/master/scripts/install.ps1 | iex
```

默认安装位置：

- Linux/macOS: `~/.local/bin/devfetch`
- Windows: `~/.local/bin/devfetch.exe`

可选环境变量：

- `DEVFETCH_REPO`：仓库地址（默认 `harkerhand/devfetch`）
- `DEVFETCH_VERSION`：版本号（默认 `latest`，也可传 `v0.1.2`）
- `DEVFETCH_INSTALL_DIR`：安装目录

示例（指定版本和目录）：

Linux/macOS：

```bash
DEVFETCH_INSTALL_DIR="$HOME/bin" \
DEVFETCH_VERSION="v0.1.2" \
curl -fsSL https://raw.githubusercontent.com/harkerhand/devfetch/master/scripts/install.sh | bash
```

Windows PowerShell：

```powershell
$env:DEVFETCH_INSTALL_DIR = "$HOME\\bin"
$env:DEVFETCH_VERSION = "v0.1.2"
irm https://raw.githubusercontent.com/harkerhand/devfetch/master/scripts/install.ps1 | iex
```

### 2) 从 Cargo 安装

从 crates.io（若已发布）：

```bash
cargo install devfetch
```

从 GitHub 仓库安装：

```bash
cargo install --git https://github.com/harkerhand/devfetch.git --locked
```

### 3) 从源码构建

```bash
cargo build --release
```

生成二进制：

- Linux/macOS: `target/release/devfetch`
- Windows: `target/release/devfetch.exe`

## 卸载

Linux/macOS（远程执行）：

```bash
curl -fsSL https://raw.githubusercontent.com/harkerhand/devfetch/master/scripts/uninstall.sh | bash
```

Windows PowerShell（远程执行）：

```powershell
irm https://raw.githubusercontent.com/harkerhand/devfetch/master/scripts/uninstall.ps1 | iex
```

自定义目录时，同样通过 `DEVFETCH_INSTALL_DIR` 传入相同路径。

## 使用示例

```bash
# 默认输出（YAML 风格）
devfetch

# JSON 输出
devfetch --json

# TOML 输出
devfetch --toml

# Markdown 输出
devfetch --markdown

# 只看系统 + 二进制工具
devfetch --system --binary

# 单项 helper（强制 JSON）
devfetch --helper Node
```

## 常见问题

- 命令找不到：请确认安装目录已加入 `PATH`。
- 首次运行较慢：工具会调用系统命令探测版本，属于正常行为。
