# 交叉编译指南

## M1 Mac 交叉编译 Windows 版本

### 方法一：使用 cross 工具（推荐）

#### 1. 安装 cross

```bash
cargo install cross
```

#### 2. 编译 Windows 版本

```bash
cd invoice_rust

# 编译 64位 Windows 版本
cross build --release --target x86_64-pc-windows-gnu
```

#### 3. 查找编译结果

编译完成后，可执行文件在：

```bash
target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
```

#### 优势

- ✅ 自动处理所有依赖
- ✅ 使用 Docker 容器隔离编译环境
- ✅ 最可靠的交叉编译方式

---

### 方法二：使用原生 cargo（轻量级）

#### 1. 安装 Windows 目标

```bash
# 添加 Windows 编译目标
rustup target add x86_64-pc-windows-gnu
```

#### 2. 安装 MinGW 工具链

```bash
# 使用 Homebrew 安装
brew install mingw-w64
```

#### 3. 编译

```bash
cd invoice_rust
cargo build --release --target x86_64-pc-windows-gnu
```

#### 4. 查找编译结果

```bash
ls -lh target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
```

#### 可能遇到的问题

如果编译失败，可能是因为某些依赖库不支持交叉编译。可以尝试：

```bash
# 设置环境变量
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar

# 然后重新编译
cargo build --release --target x86_64-pc-windows-gnu
```

---

### 方法三：GitHub Actions 自动编译（最佳实践）

创建 `.github/workflows/release.yml`：

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-pc-windows-gnu
            artifact_name: invoice-extractor.exe
            asset_name: invoice-extractor-windows-x64.exe
          
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: invoice-extractor
            asset_name: invoice-extractor-macos-intel
          
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact_name: invoice-extractor
            asset_name: invoice-extractor-macos-m1
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
        override: true
    
    - name: Build
      run: cargo build --release --target ${{ matrix.target }}
    
    - name: Upload binaries
      uses: actions/upload-artifact@v3
      with:
        name: ${{ matrix.asset_name }}
        path: target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
```

使用方式：

```bash
# 创建并推送标签
git tag v0.3.0
git push origin v0.3.0

# GitHub Actions 会自动编译所有平台版本
```

---

## 支持的编译目标

### Windows

```bash
# 64位 (推荐)
x86_64-pc-windows-gnu      # MinGW 工具链，更容易交叉编译
x86_64-pc-windows-msvc     # MSVC 工具链，需要 Windows SDK

# 32位
i686-pc-windows-gnu
i686-pc-windows-msvc
```

### macOS

```bash
# Intel Mac
x86_64-apple-darwin

# M1/M2 Mac (Apple Silicon)
aarch64-apple-darwin
```

### Linux

```bash
# 64位
x86_64-unknown-linux-gnu
x86_64-unknown-linux-musl  # 静态链接，更好的兼容性

# ARM64
aarch64-unknown-linux-gnu
```

---

## 完整的跨平台编译脚本

创建 `build-all.sh`：

```bash
#!/bin/bash

echo "开始跨平台编译..."

# 检查是否安装了 cross
if ! command -v cross &> /dev/null; then
    echo "安装 cross..."
    cargo install cross
fi

# 编译目标列表
TARGETS=(
    "x86_64-pc-windows-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-unknown-linux-gnu"
)

# 编译每个目标
for target in "${TARGETS[@]}"; do
    echo "正在编译 $target..."
    cross build --release --target $target
    
    if [ $? -eq 0 ]; then
        echo "✅ $target 编译成功"
    else
        echo "❌ $target 编译失败"
    fi
done

echo ""
echo "编译结果："
echo "-------------------------------------"

# 显示编译结果
for target in "${TARGETS[@]}"; do
    if [ -f "target/$target/release/invoice-extractor.exe" ]; then
        size=$(ls -lh "target/$target/release/invoice-extractor.exe" | awk '{print $5}')
        echo "$target: $size"
    elif [ -f "target/$target/release/invoice-extractor" ]; then
        size=$(ls -lh "target/$target/release/invoice-extractor" | awk '{print $5}')
        echo "$target: $size"
    fi
done

echo "-------------------------------------"
echo "完成！"
```

使用方式：

```bash
chmod +x build-all.sh
./build-all.sh
```

---

## 打包发布

### Windows 版本打包

```bash
# 创建发布目录
mkdir -p release/windows

# 复制可执行文件
cp target/x86_64-pc-windows-gnu/release/invoice-extractor.exe release/windows/

# 复制配置文件示例
cp config.example.toml release/windows/config.toml

# 创建 README
cat > release/windows/README.txt << 'EOF'
发票识别工具 - Windows 版本

使用方法：
1. 双击 invoice-extractor.exe 启动程序
2. 如果遇到缺少 DLL 的问题，请安装 Visual C++ Redistributable
3. 配置文件 config.toml 可以调整线程数

支持：
- Windows 10/11 (64位)
EOF

# 打包成 zip
cd release
zip -r invoice-extractor-windows-v0.3.0.zip windows/
cd ..
```

### macOS 版本打包

```bash
# 创建 macOS 应用包
mkdir -p release/macos/InvoiceExtractor.app/Contents/MacOS
mkdir -p release/macos/InvoiceExtractor.app/Contents/Resources

# 复制可执行文件
cp target/aarch64-apple-darwin/release/invoice-extractor \
   release/macos/InvoiceExtractor.app/Contents/MacOS/

# 创建 Info.plist
cat > release/macos/InvoiceExtractor.app/Contents/Info.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>发票识别工具</string>
    <key>CFBundleExecutable</key>
    <string>invoice-extractor</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.invoice-extractor</string>
    <key>CFBundleVersion</key>
    <string>0.3.0</string>
</dict>
</plist>
EOF

# 打包成 dmg
hdiutil create -volname "发票识别工具" -srcfolder release/macos -ov -format UDZO \
    release/invoice-extractor-macos-v0.3.0.dmg
```

---

## 测试交叉编译的可执行文件

### 在 macOS 上测试 Windows 可执行文件

虽然不能直接运行，但可以检查文件格式：

```bash
# 安装 file 命令（如果没有）
brew install file

# 检查 Windows 可执行文件
file target/x86_64-pc-windows-gnu/release/invoice-extractor.exe

# 应该显示：
# PE32+ executable (console) x86-64, for MS Windows
```

### 使用 Wine 测试（可选）

```bash
# 安装 Wine
brew install --cask wine-stable

# 运行 Windows 程序
wine target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
```

---

## 常见问题

### Q1: 编译失败，提示找不到工具链

**A**: 确保安装了必要的工具：

```bash
# 重新安装 MinGW
brew reinstall mingw-w64

# 检查是否安装成功
which x86_64-w64-mingw32-gcc
```

### Q2: 编译的 Windows 程序在目标机器上无法运行

**A**: 可能缺少运行时库，有两种解决方案：

1. 静态链接（推荐）：

```bash
# 在 .cargo/config.toml 添加
[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-args=-static"]
```

1. 或者提供 MinGW 运行时 DLL

### Q3: egui 界面程序交叉编译失败

**A**: GUI 程序交叉编译可能遇到更多问题，建议使用 `cross` 工具或 GitHub Actions。

### Q4: 文件太大

**A**: 使用 UPX 压缩：

```bash
# 安装 UPX
brew install upx

# 压缩可执行文件
upx --best --lzma target/x86_64-pc-windows-gnu/release/invoice-extractor.exe

# 可以减小 50-70% 的体积
```

---

## 快速命令参考

```bash
# 安装 cross（一次性）
cargo install cross

# 编译 Windows 版本
cross build --release --target x86_64-pc-windows-gnu

# 编译 macOS Intel 版本
cross build --release --target x86_64-apple-darwin

# 编译 macOS M1 版本（在 M1 Mac 上）
cargo build --release --target aarch64-apple-darwin

# 编译 Linux 版本
cross build --release --target x86_64-unknown-linux-gnu

# 查看所有已安装的目标
rustup target list --installed

# 添加新目标
rustup target add <target-name>
```

---

## 推荐工作流

### 开发阶段

```bash
# 本地开发和测试（M1 Mac）
cargo run

# 快速编译检查
cargo check
```

### 发布阶段

```bash
# 使用 cross 编译所有平台版本
./build-all.sh

# 或使用 GitHub Actions 自动编译
git tag v0.3.0
git push origin v0.3.0
```

---

**推荐使用 `cross` 工具进行交叉编译，它能处理大部分复杂依赖问题！**
