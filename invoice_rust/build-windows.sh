#!/bin/bash

echo "==================================="
echo "  发票识别工具 - Windows 交叉编译"
echo "==================================="
echo ""

# 检查是否安装了 cross
if ! command -v cross &> /dev/null; then
    echo "📦 cross 未安装，正在安装..."
    cargo install cross
    echo ""
fi

# 开始编译
echo "🔨 开始编译 Windows 版本..."
echo "目标: x86_64-pc-windows-gnu (64位 Windows)"
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
    echo "1. cross 未正确安装"
    echo "2. Docker 未运行（cross 需要 Docker）"
    echo ""
    echo "解决方案："
    echo "- 确保 Docker Desktop 已安装并运行"
    echo "- 运行: cargo install cross --force"
    exit 1
fi

echo ""
echo "完成！"
