//! 错误类型定义
//!
//! 使用 anyhow 提供上下文丰富的错误处理

use thiserror::Error;

/// rg-cli 错误类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 无效的正则表达式
    #[error("Invalid regex pattern '{0}': {1}")]
    InvalidRegex(String, #[source] regex::Error),

    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 路径解析错误
    #[error("Path error: {0}")]
    Path(String),

    /// 搜索被中断
    #[error("Search interrupted")]
    Interrupted,

    /// 权限错误
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

impl From<regex::Error> for AppError {
    fn from(e: regex::Error) -> Self {
        AppError::InvalidRegex("".to_string(), e)
    }
}

/// 搜索结果类型
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 文件路径
    pub path: String,
    /// 行号（0 表示文件名模式）
    pub line_number: Option<u32>,
    /// 匹配的行内容
    pub line: String,
    /// 匹配的列位置
    pub start_column: usize,
    /// 匹配的结束列位置
    pub end_column: usize,
    /// 是否为二进制文件
    pub is_binary: bool,
}

impl SearchResult {
    pub fn new(
        path: String,
        line_number: Option<u32>,
        line: String,
        start_column: usize,
        end_column: usize,
    ) -> Self {
        Self {
            path,
            line_number,
            line,
            start_column,
            end_column,
            is_binary: false,
        }
    }

    pub fn binary(path: String, line_number: Option<u32>, line: String) -> Self {
        Self {
            path,
            line_number,
            line,
            start_column: 0,
            end_column: 0,
            is_binary: true,
        }
    }
}
