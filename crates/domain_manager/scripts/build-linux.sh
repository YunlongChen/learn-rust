#!/bin/bash
# Linux 打包脚本
# 用于构建 Linux 平台的 domain_manager 应用程序

set -e  # 遇到错误时退出

# 默认参数
BUILD_TYPE="release"
CLEAN=false
TARGET="x86_64-unknown-linux-gnu"

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        --help)
            echo "用法: $0 [选项]"
            echo "选项:"
            echo "  --debug     构建调试版本（默认为release）"
            echo "  --clean     清理构建缓存"
            echo "  --target    指定目标平台（默认为x86_64-unknown-linux-gnu）"
            echo "  --help      显示此帮助信息"
            exit 0
            ;;
        *)
            echo "未知选项: $1"
            exit 1
            ;;
    esac
done

# 获取脚本所在目录的父目录（项目根目录）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "开始构建 Linux 版本..."
echo "项目根目录: $PROJECT_ROOT"
echo "构建类型: $BUILD_TYPE"
echo "目标平台: $TARGET"

# 清理构建缓存（如果指定）
if [ "$CLEAN" = true ]; then
    echo "清理构建缓存..."
    cargo clean
fi

# 检查 Rust 工具链
echo "检查 Rust 工具链..."
rustc --version
cargo --version

# 添加目标平台（如果尚未添加）
echo "确保目标平台已安装..."
rustup target add "$TARGET"

# 安装必要的系统依赖（Ubuntu/Debian）
if command -v apt-get >/dev/null 2>&1; then
    echo "检查系统依赖..."
    # 检查是否需要安装依赖
    MISSING_DEPS=()
    
    # 检查 pkg-config
    if ! command -v pkg-config >/dev/null 2>&1; then
        MISSING_DEPS+=("pkg-config")
    fi
    
    # 检查开发库
    if ! pkg-config --exists fontconfig; then
        MISSING_DEPS+=("libfontconfig1-dev")
    fi
    
    if ! pkg-config --exists freetype2; then
        MISSING_DEPS+=("libfreetype6-dev")
    fi
    
    if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
        echo "需要安装以下依赖: ${MISSING_DEPS[*]}"
        echo "请运行: sudo apt-get update && sudo apt-get install ${MISSING_DEPS[*]}"
        echo "或者使用 --skip-deps 跳过依赖检查"
    fi
fi

# 构建应用程序
echo "开始编译..."
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release --target "$TARGET"
    BINARY_PATH="../../target/$TARGET/release/domain_manager"
else
    cargo build --target "$TARGET"
    BINARY_PATH="../../target/$TARGET/debug/domain_manager"
fi

# 检查构建是否成功
if [ -f "$BINARY_PATH" ]; then
    echo "构建成功！"
    echo "可执行文件位置: $BINARY_PATH"
    
    # 显示文件信息
    FILE_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
    echo "文件大小: $FILE_SIZE"
    
    # 创建发布目录
    RELEASE_DIR="release/linux-x64"
    mkdir -p "$RELEASE_DIR"
    
    # 复制可执行文件
    cp "$BINARY_PATH" "$RELEASE_DIR/domain_manager"
    chmod +x "$RELEASE_DIR/domain_manager"
    
    # 复制资源文件
    if [ -d "resources" ]; then
        echo "复制资源文件..."
        cp -r "resources" "$RELEASE_DIR/"
    fi
    
    # 复制配置文件
    if [ -d "config" ]; then
        echo "复制配置文件..."
        cp -r "config" "$RELEASE_DIR/"
    fi
    
    # 复制本地化文件
    if [ -d "locales" ]; then
        echo "复制本地化文件..."
        cp -r "locales" "$RELEASE_DIR/"
    fi
    
    # 创建启动脚本
    cat > "$RELEASE_DIR/start.sh" << 'EOF'
#!/bin/bash
# Domain Manager 启动脚本

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "启动 Domain Manager..."

# 设置环境变量
export RUST_LOG=info

# 启动应用程序
./domain_manager "$@"
EOF
    
    chmod +x "$RELEASE_DIR/start.sh"
    
    # 创建 .desktop 文件（Linux 桌面快捷方式）
    cat > "$RELEASE_DIR/domain-manager.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Domain Manager
Comment=域名和DNS管理工具
Exec=$PWD/$RELEASE_DIR/domain_manager
Icon=$PWD/$RELEASE_DIR/resources/icons/app-icon.png
Terminal=false
Categories=Network;Utility;
EOF
    
    echo "打包完成！"
    echo "发布目录: $RELEASE_DIR"
    echo ""
    echo "使用方法:"
    echo "  直接运行: ./$RELEASE_DIR/domain_manager"
    echo "  使用启动脚本: ./$RELEASE_DIR/start.sh"
    echo "  安装桌面快捷方式: cp $RELEASE_DIR/domain-manager.desktop ~/.local/share/applications/"
    
else
    echo "构建失败！"
    exit 1
fi

echo "Linux 构建脚本执行完成。"