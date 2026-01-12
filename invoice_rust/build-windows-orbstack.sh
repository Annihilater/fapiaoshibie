#!/bin/bash

echo "==================================="
echo "  发票识别工具 - Windows 交叉编译"
echo "  (OrbStack 优化版本)"
echo "==================================="
echo ""

# 检查 OrbStack 是否运行
if ! command -v orbctl &> /dev/null; then
    echo "⚠️  未检测到 OrbStack，请先安装 OrbStack"
    echo "   下载地址: https://orbstack.dev"
    exit 1
fi

echo "✅ 检测到 OrbStack"
echo ""

# 检查 Docker 兼容模式是否可用
if ! docker ps &> /dev/null; then
    echo "⚠️  OrbStack 的 Docker 兼容模式未启动"
    echo "   请在 OrbStack 中启用 Docker 功能"
    exit 1
fi

echo "✅ Docker 兼容模式已启用"
echo ""

# 检查是否安装了 cross
if ! command -v cross &> /dev/null; then
    echo "📦 cross 未安装，正在安装..."
    cargo install cross --git https://github.com/cross-rs/cross
    echo ""
fi

echo "✅ cross 已安装"
echo ""

# 设置环境变量以适配 OrbStack
export CROSS_CONTAINER_ENGINE=docker
export CROSS_REMOTE_SKIP_LOCAL_DOCKER_VERSION_CHECK=1

echo "🔨 开始编译 Windows 版本..."
echo "目标: x86_64-pc-windows-gnu (64位 Windows)"
echo "容器引擎: OrbStack (Docker 兼容模式)"
echo ""

# 使用 cross 编译，忽略主机工具链检查
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
- 如遇到缺少 DLL，请安装 Visual C++ Redistributable

编译环境：
- 使用 OrbStack 在 M1 Mac 上交叉编译

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
    echo "可能的原因："
    echo "1. OrbStack 容器启动失败"
    echo "2. 网络问题，无法下载 Docker 镜像"
    echo "3. cross 版本不兼容"
    echo ""
    echo "解决方案："
    echo "- 重启 OrbStack"
    echo "- 检查网络连接"
    echo "- 运行: cargo install cross --git https://github.com/cross-rs/cross --force"
    echo "- 查看详细错误信息并搜索解决方案"
    exit 1
fi

echo ""
echo "完成！"
