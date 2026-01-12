# 快速修复指南 - OrbStack 交叉编译

## 🎯 问题诊断

你遇到的问题：
1. ❌ Rust 版本过低（1.89.0 < 1.92.0）
2. ❌ cross 尝试安装不兼容的 Linux 工具链

## ⚡ 最简单的解决方案

### 方案 A: 使用简化脚本（推荐）

我创建了一个新脚本，会自动更新 Rust 并使用正确的配置：

```bash
cd /Users/ziji/github/fapiaoshibie/invoice_rust
./build-windows-simple.sh
```

这个脚本会：
1. ✅ 自动更新 Rust 到最新版本
2. ✅ 安装兼容的 cross 版本
3. ✅ 配置正确的环境变量
4. ✅ 自动编译和打包

### 方案 B: 手动操作（如果脚本失败）

```bash
# 1. 更新 Rust
rustup update stable
rustup default stable

# 2. 验证版本
rustc --version  # 应该显示 1.92.0 或更高

# 3. 安装 cross（使用稳定版）
cargo install cross --version 0.2.5

# 4. 设置环境变量
export CROSS_CONTAINER_ENGINE=docker
export CROSS_REMOTE_SKIP_LOCAL_DOCKER_VERSION_CHECK=1

# 5. 编译
cross build --release --target x86_64-pc-windows-gnu
```

---

## 🔧 详细步骤说明

### 步骤 1: 更新 Rust

```bash
# 更新到最新稳定版
rustup update stable

# 设置为默认
rustup default stable

# 验证版本
rustc --version
```

**预期输出：**
```
rustc 1.92.0 (或更高版本)
```

### 步骤 2: 重装 cross

```bash
# 卸载旧版本
cargo uninstall cross

# 安装稳定版（不使用 git）
cargo install cross --version 0.2.5
```

### 步骤 3: 预拉取 Docker 镜像

这样可以避免首次编译时的等待：

```bash
docker pull ghcr.io/cross-rs/x86_64-pc-windows-gnu:latest
```

### 步骤 4: 编译

```bash
cd /Users/ziji/github/fapiaoshibie/invoice_rust

# 设置环境变量
export CROSS_CONTAINER_ENGINE=docker
export CROSS_REMOTE_SKIP_LOCAL_DOCKER_VERSION_CHECK=1

# 开始编译
cross build --release --target x86_64-pc-windows-gnu
```

---

## 📊 预期输出

成功的编译过程应该是这样的：

```
✅ Rust 已更新到最新版本
✅ cross 已安装
🔨 开始编译...

[info]: build container ghcr.io/cross-rs/x86_64-pc-windows-gnu:latest
   Compiling libc v0.2.180
   Compiling proc-macro2 v1.0.105
   ...
   Compiling invoice-extractor v0.1.0
    Finished `release` profile [optimized] target(s) in 5m 23s

✅ 编译成功！
📁 target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
```

---

## 🐛 如果还是失败...

### 检查 1: OrbStack 是否正常

```bash
# 测试 Docker 功能
docker ps

# 测试拉取镜像
docker pull hello-world
docker run hello-world
```

### 检查 2: 网络连接

```bash
# 测试是否能访问 GitHub Container Registry
curl -I https://ghcr.io

# 如果超时，可能需要配置代理
```

### 检查 3: 磁盘空间

```bash
# 检查可用空间（至少需要 10GB）
df -h

# 清理 Docker 缓存
docker system prune -a
```

### 检查 4: Cross.toml 配置

确保 `Cross.toml` 文件存在且内容正确：

```toml
[build]
[target.x86_64-pc-windows-gnu]
image = "ghcr.io/cross-rs/x86_64-pc-windows-gnu:latest"

[build.env]
passthrough = [
    "CARGO_HOME",
    "RUSTUP_HOME",
]
```

---

## 💡 替代方案

如果 cross 完全不工作，可以尝试：

### 方案 1: 使用 GitHub Actions（推荐）

在 GitHub 上设置自动编译，不需要本地环境：

```yaml
# .github/workflows/build-windows.yml
name: Build Windows

on:
  push:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: x86_64-pc-windows-gnu
    - uses: actions-rs/cargo@v1
      with:
        use-cross: true
        command: build
        args: --release --target x86_64-pc-windows-gnu
    - uses: actions/upload-artifact@v3
      with:
        name: windows-binary
        path: target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
```

### 方案 2: 原生交叉编译（不推荐，可能失败）

```bash
# 安装 MinGW
brew install mingw-w64

# 添加目标
rustup target add x86_64-pc-windows-gnu

# 尝试编译（可能因为依赖问题失败）
cargo build --release --target x86_64-pc-windows-gnu
```

### 方案 3: 使用 Windows 虚拟机

如果有 Parallels Desktop 或 VMware：
1. 创建 Windows 虚拟机
2. 在虚拟机中安装 Rust
3. 直接在 Windows 中编译

---

## ⏱️ 预期时间

| 操作 | 首次 | 后续 |
|-----|------|------|
| 更新 Rust | 5分钟 | - |
| 安装 cross | 2分钟 | - |
| 下载 Docker 镜像 | 5-10分钟 | - |
| 编译项目 | 5-10分钟 | 2-3分钟 |
| **总计** | **15-25分钟** | **2-3分钟** |

---

## ✅ 成功检查清单

编译成功后，验证：

```bash
# 1. 文件存在
ls -lh target/x86_64-pc-windows-gnu/release/invoice-extractor.exe

# 2. 文件类型正确
file target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
# 应显示: PE32+ executable (console) x86-64, for MS Windows

# 3. 文件大小合理（约 9-10MB）
du -h target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
```

---

## 🎉 快速开始

**最简单的方式：**

```bash
cd /Users/ziji/github/fapiaoshibie/invoice_rust
./build-windows-simple.sh
```

按照提示操作，脚本会处理所有事情！

---

**遇到问题？** 把错误信息发给我，我会帮你解决！
