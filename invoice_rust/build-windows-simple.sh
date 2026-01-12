#!/bin/bash

echo "==================================="
echo "  发票识别工具 - Windows 交叉编译"
echo "  (简化版本 - 适用于 OrbStack)"
echo "==================================="
echo ""

# 步骤 1: 更新 Rust
echo "📦 步骤 1: 检查 Rust 版本..."
RUST_VERSION=$(rustc --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
echo "当前 Rust 版本: $RUST_VERSION"

REQUIRED_VERSION="1.92.0"
if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" = "$REQUIRED_VERSION" ]; then 
    echo "✅ Rust 版本满足要求"
else
    echo "⚠️  Rust 版本过低，正在更新..."
    echo ""
    rustup update stable
    rustup default stable
    echo ""
    echo "✅ Rust 已更新到最新版本"
fi

echo ""

# 步骤 2: 安装稳定版 cross
echo "📦 步骤 2: 安装 cross (使用 crates.io 稳定版)..."
cargo install cross --version 0.2.5

echo ""

# 步骤 3: 配置环境
echo "🔧 步骤 3: 配置环境变量..."
export CROSS_CONTAINER_ENGINE=docker
export CROSS_REMOTE_SKIP_LOCAL_DOCKER_VERSION_CHECK=1

echo "✅ 环境配置完成"
echo ""

# 步骤 4: 开始编译
echo "🔨 步骤 4: 开始编译 Windows 版本..."
echo "目标: x86_64-pc-windows-gnu (64位 Windows)"
echo "容器引擎: OrbStack"
echo ""

cross build --release --target x86_64-pc-windows-gnu

# 检查编译结果
if [ $? -eq 0 ]; then
    echo ""
    echo "✅ 编译成功！"
    echo ""
    echo "📁 可执行文件位置："
    echo "   target/x86_64-pc-windows-gnu/release/invoice-extractor.exe"
    echo ""
    
    # 显示文件大小
    if [ -f "target/x86_64-pc-windows-gnu/release/invoice-extractor.exe" ]; then
        size=$(ls -lh target/x86_64-pc-windows-gnu/release/invoice-extractor.exe | awk '{print $5}')
        echo "📊 文件大小: $size"
        echo ""
        
        # 验证文件类型
        echo "🔍 验证文件类型:"
        file target/x86_64-pc-windows-gnu/release/invoice-extractor.exe
        echo ""
    fi
    
    # 询问是否打包
    echo "是否创建发布包？(y/n)"
    read -r response
    
    if [[ "$response" =~ ^[Yy]$ ]]; then
        echo ""
        echo "📦 正在打包..."
        
        # 创建发布目录
        rm -rf release/windows
        mkdir -p release/windows
        
        # 复制文件
        cp target/x86_64-pc-windows-gnu/release/invoice-extractor.exe release/windows/
        cp config.example.toml release/windows/config.toml
        
        # 创建说明文件
        cat > release/windows/README.txt << 'EOF'
发票识别工具 Windows 版本 v0.3.0
=====================================

功能特点：
✅ 多线程并行处理（32线程）
✅ 实时进度显示
✅ 自动识别发票信息
✅ 生成 Excel 清单

使用方法：
1. 双击 invoice-extractor.exe 启动程序
2. 选择发票目录
3. 点击"开始识别"按钮
4. 查看实时进度和结果

配置文件：
- config.toml: 可以调整线程数
- 默认 32 线程，可根据电脑配置调整

系统要求：
- Windows 10/11 (64位)

编译信息：
- 在 M1 Mac + OrbStack 上交叉编译
- 使用 Rust + cross 工具链

版本：v0.3.0
日期：2026-01-12
EOF
        
        # 打包
        cd release
        zip -r invoice-extractor-windows-v0.3.0.zip windows/
        cd ..
        
        echo ""
        echo "✅ 打包完成！"
        echo "📦 发布包位置: release/invoice-extractor-windows-v0.3.0.zip"
        echo ""
        echo "可以将此 zip 文件发送给 Windows 用户使用。"
    fi
else
    echo ""
    echo "❌ 编译失败！"
    echo ""
    echo "请尝试以下步骤："
    echo "1. 检查 Docker/OrbStack 是否正常运行"
    echo "2. 运行: docker pull ghcr.io/cross-rs/x86_64-pc-windows-gnu:latest"
    echo "3. 如果还是失败，查看完整错误信息"
    exit 1
fi

echo ""
echo "完成！"
