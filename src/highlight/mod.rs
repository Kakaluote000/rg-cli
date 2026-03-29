//! 颜色高亮输出模块
//!
//! 负责在终端中显示带颜色的搜索结果

use crate::error::SearchResult;

/// ANSI 转义码
pub mod ansi {
    /// 重置
    pub const RESET: &str = "\x1b[0m";
    /// 粗体
    pub const BOLD: &str = "\x1b[1m";
    /// 红色（用于文件名）
    pub const RED: &str = "\x1b[31m";
    /// 绿色（用于匹配）
    pub const GREEN: &str = "\x1b[32m";
    /// 黄色
    pub const YELLOW: &str = "\x1b[33m";
    /// 蓝色
    pub const BLUE: &str = "\x1b[34m";
    /// 洋红色
    pub const MAGENTA: &str = "\x1b[35m";
    /// 青色
    pub const CYAN: &str = "\x1b[36m";
    /// 白色
    pub const WHITE: &str = "\x1b[37m";
    /// 浅灰色
    pub const BRIGHT_BLACK: &str = "\x1b[90m";
    /// 深灰色（用于行号）
    pub const BRIGHT_BLACK_ALT: &str = "\x1b[38;5;245m";
    /// 反显（用于高亮匹配）
    pub const REVERSE: &str = "\x1b[7m";
}

/// 格式化匹配项
pub struct Highlighter {
    use_color: bool,
}

impl Highlighter {
    pub fn new(use_color: bool) -> Self {
        Self { use_color }
    }

    /// 高亮单个匹配
    pub fn highlight_match(&self, text: &str, start: usize, end: usize) -> String {
        if !self.use_color {
            return text.to_string();
        }

        let before = &text[..start];
        let match_text = &text[start..end];
        let after = &text[end..];

        format!(
            "{}{}{}{}{}",
            before,
            ansi::REVERSE,  // 反显高亮
            match_text,
            ansi::RESET,
            after
        )
    }

    /// 格式化路径
    pub fn format_path(&self, path: &str) -> String {
        if self.use_color {
            format!("{}{}{}", ansi::BLUE, path, ansi::RESET)
        } else {
            path.to_string()
        }
    }

    /// 格式化行号
    pub fn format_line_number(&self, line_no: u32) -> String {
        if self.use_color {
            format!(
                "{}{:>6}{}: {}",
                ansi::BRIGHT_BLACK_ALT,
                line_no,
                ansi::RESET,
                ansi::RESET
            )
        } else {
            format!("{:>6}: ", line_no)
        }
    }

    /// 格式化分隔符
    pub fn format_separator(&self) -> String {
        if self.use_color {
            format!("{}:{}{}", ansi::RED, ansi::RESET, " ")
        } else {
            ":".to_string()
        }
    }
}

/// 打印搜索结果
pub fn print_results(results: &[SearchResult], use_color: bool) -> Result<(), std::io::Error> {
    let highlighter = Highlighter::new(use_color);

    // 按文件分组输出
    let mut current_file = String::new();

    for result in results {
        // 如果换了文件，打印文件名
        if result.path != current_file {
            if !current_file.is_empty() {
                println!();
            }
            print!("{}", highlighter.format_path(&result.path));
            println!();
            current_file = result.path.clone();
        }

        // 打印行号和内容
        if let Some(line_no) = result.line_number {
            print!("{}", highlighter.format_line_number(line_no));
        }

        // 高亮匹配
        let highlighted = highlighter.highlight_match(
            &result.line,
            result.start_column,
            result.end_column,
        );
        println!("{}", highlighted);
    }

    Ok(())
}

/// 禁用颜色的版本（用于测试）
#[cfg(test)]
pub fn print_results_plain(results: &[SearchResult]) -> Result<(), std::io::Error> {
    for result in results {
        if let Some(line_no) = result.line_number {
            println!("{}:{:>6}: {}", result.path, line_no, result.line);
        } else {
            println!("{}", result.path);
        }
    }
    Ok(())
}
