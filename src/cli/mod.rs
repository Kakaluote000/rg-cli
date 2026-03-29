//! 命令行参数解析模块
//!
//! 使用 clap derive 模式定义所有 CLI 参数

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// rg-cli - 快速文件搜索工具
///
/// 一个类 ripgrep 的命令行搜索工具，支持正则表达式、并发搜索、颜色高亮
#[derive(Parser, Debug)]
#[command(
    name = "rg",
    version,
    author,
    about = "Search for patterns in files",
    long_about = None,
    disable_help_flag = false,
)]
pub struct Cli {
    /// 搜索模式（正则表达式）
    #[arg(value_name = "PATTERN")]
    pub pattern: String,

    /// 搜索路径（默认为当前目录）
    #[arg(value_name = "PATH", default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// 忽略大小写
    #[arg(short, long)]
    pub ignore_case: bool,

    /// 递归遍历目录
    #[arg(short, long, default_value_t = true)]
    pub recursive: bool,

    /// 最大深度（0 表示无限制）
    #[arg(short = 'd', long, default_value_t = u32::MAX)]
    pub max_depth: u32,

    /// 排除的文件或目录（可多次指定）
    #[arg(short, long, value_name = "PATTERN")]
    pub exclude: Vec<String>,

    /// 只搜索文件名，不显示行内容
    #[arg(short = 'l', long)]
    pub files_with_matches: bool,

    /// 只显示文件名，不搜索内容
    #[arg(short = 'L', long)]
    pub files_without_matches: bool,

    /// 显示匹配的行号
    #[arg(short = 'n', long)]
    pub line_numbers: bool,

    /// 显示匹配的总数（不显示具体行）
    #[arg(short = 'c', long)]
    pub count: bool,

    /// 每个文件最多显示的匹配数
    #[arg(long, default_value_t = u32::MAX)]
    pub max_count: u32,

    /// 显示颜色高亮
    #[arg(
        long,
        value_enum,
        default_value = "auto",
        env = "RG_COLOR",
        value_name = "WHEN"
    )]
    pub color: ColorChoice,

    /// 显示性能统计信息
    #[arg(long)]
    pub stats: bool,

    /// 不显示性能统计信息
    #[arg(long, hide = true)]
    pub no_stats: bool,

    /// 跟随符号链接
    #[arg(long)]
    pub follow: bool,

    /// 搜索隐藏文件
    #[arg(long)]
    pub hidden: bool,

    /// 显示文件名（无论是否有匹配）
    #[arg(short = 'a', long)]
    pub with_filename: bool,

    /// 二进制文件处理方式
    #[arg(long, value_enum, default_value = "no")]
    pub binary: BinaryMode,

    /// 安静模式（只返回退出码）
    #[arg(short, long)]
    pub quiet: bool,

    /// 启用详细输出
    #[arg(short, long)]
    pub verbose: bool,

    /// 最大并发数（0 表示使用 CPU 核数）
    #[arg(long, default_value_t = 0)]
    pub jobs: u32,
}

impl Cli {
    /// 返回颜色配置
    pub fn color(&self) -> bool {
        match self.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => {
                std::env::var("FORCE_COLOR").is_ok()
                    || std::io::IsTerminal::is_terminal(&std::io::stdout())
            }
        }
    }
}

/// 颜色输出配置
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum ColorChoice {
    /// 始终使用颜色
    Always,
    /// 从不使用颜色
    Never,
    /// 自动检测（默认）
    #[default]
    Auto,
}

/// 二进制文件处理模式
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum BinaryMode {
    /// 不搜索二进制文件
    #[default]
    No,
    /// 搜索二进制文件（可能显示乱码）
    Yes,
    /// 用 <BINARY> 标记替换二进制内容
    WithPlaceholder,
}
