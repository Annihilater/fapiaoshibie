# 快速交叉编译指南（M1 Mac → Windows）

## 🎯 最简单的方法（推荐）

### 步骤 1: 安装 Docker Desktop

交叉编译需要 Docker：

1. 下载安装：<https://www.docker.com/products/docker-desktop/>
2. 启动 Docker Desktop
3. 确保 Docker 正在运行

### 步骤 2: 运行编译脚本

```bash
cd invoice_rust
./build-windows.sh
```

就这么简单！脚本会自动：

- ✅ 安装 `cross` 工具（如果未安装）
- ✅ 交叉编译 Windows 版本
- ✅ 询问是否打包发布
- ✅ 创建 zip 压缩包

### 步骤 3: 获取结果

编译完成后，可执行文件在：

```
target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
```

如果选择打包，会生成：

```
release/invoice-extractor-windows-v0.3.0.zip
```

---

## 🔧 手动编译步骤

如果你想手动控制每个步骤：

### 1. 安装 cross

```bash
cargo install cross
```

### 2. 确保 Docker 运行中

```bash
# 检查 Docker 是否运行
docker ps
```

### 3. 编译

```bash
cd invoice_rust
cross build --release --target x86_64-pc-windows-gnu
```

### 4. 查找结果

```bash
ls -lh target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
```

---

## 📦 打包发布

### 创建完整的发布包

```bash
# 创建目录
mkdir -p release/windows

# 复制文件
cp target/x86_64-pc-windows-gnu/release/invoice-extractor.exe release/windows/
cp config.example.toml release/windows/config.toml

# 创建说明
echo "发票识别工具 Windows 版本
使用方法：双击 invoice-extractor.exe 启动
配置文件：config.toml" > release/windows/README.txt

# 打包
cd release
zip -r invoice-extractor-windows-v0.3.0.zip windows/
```

---

## ❓ 常见问题

### Q: Docker 必须安装吗？

**A**: 是的。`cross` 使用 Docker 容器来提供完整的交叉编译环境，这是最可靠的方式。

### Q: 可以不用 Docker 吗？

**A**: 可以尝试原生交叉编译，但不推荐，步骤复杂且容易出错：

```bash
# 安装 MinGW（不推荐，仅供参考）
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu

# 设置环境变量
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++

# 尝试编译（可能失败）
cargo build --release --target x86_64-pc-windows-gnu
```

### Q: 编译需要多长时间？

**A**:

- 首次编译：5-10分钟（需要下载 Docker 镜像和编译依赖）
- 后续编译：1-3分钟

### Q: 编译的 Windows 程序能在所有 Windows 上运行吗？

**A**: 可以在 Windows 10/11 64位系统上运行。如果遇到缺少 DLL 的问题，用户需要安装 Visual C++ Redistributable。

### Q: 如何测试编译的 Windows 程序？

**A**:

1. 最好在真实 Windows 机器上测试
2. 或使用虚拟机（Parallels Desktop / VMware）
3. 或使用 Wine（但可能有兼容性问题）

---

## 🚀 编译其他平台

### Windows 32位

```bash
cross build --release --target i686-pc-windows-gnu
```

### Linux

```bash
cross build --release --target x86_64-unknown-linux-gnu
```

### macOS Intel

```bash
cross build --release --target x86_64-apple-darwin
```

---

## 📋 完整工作流示例

```bash
# 1. 确保 Docker 运行
open -a Docker

# 2. 进入项目目录
cd invoice_rust

# 3. 运行编译脚本
./build-windows.sh

# 4. 等待编译完成
# 首次编译会下载 Docker 镜像，需要几分钟

# 5. 获取发布包
# release/invoice-extractor-windows-v0.3.0.zip

# 6. 分享给 Windows 用户
# 可以通过邮件、网盘等方式发送
```

---

## ✅ 检查清单

编译前确认：

- [ ] Docker Desktop 已安装
- [ ] Docker 正在运行
- [ ] 网络连接正常（首次需要下载镜像）
- [ ] 磁盘空间充足（至少 5GB）

编译后验证：

- [ ] 可执行文件已生成
- [ ] 文件大小约 9-10MB
- [ ] 使用 `file` 命令确认文件类型

```bash
file target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
# 应显示: PE32+ executable (console) x86-64, for MS Windows
```

---

## 🎉 快速开始

最快的方式：

```bash
# 安装 Docker Desktop，然后运行：
cd invoice_rust && ./build-windows.sh
```

就这么简单！✨
