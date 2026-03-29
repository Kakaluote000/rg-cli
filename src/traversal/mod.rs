//! 文件遍历模块
//!
//! 负责递归遍历目录，支持排除规则、深度控制、符号链接跟随等

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// 文件遍历器配置
#[derive(Debug, Clone)]
pub struct Traverser {
    /// 最大深度（0 表示无限制）
    pub max_depth: u32,
    /// 排除规则列表
    pub exclude_patterns: Vec<String>,
    /// 是否跟随符号链接
    pub follow_links: bool,
    /// 是否包含隐藏文件
    pub include_hidden: bool,
    /// 二进制文件处理模式
    pub binary_mode: super::cli::BinaryMode,
}

impl Traverser {
    pub fn new(
        max_depth: u32,
        exclude_patterns: Vec<String>,
        follow_links: bool,
        include_hidden: bool,
        binary_mode: super::cli::BinaryMode,
    ) -> Self {
        Self {
            max_depth,
            exclude_patterns,
            follow_links,
            include_hidden,
            binary_mode,
        }
    }

    /// 遍历路径列表，返回所有匹配的文件
    pub fn traverse(&self, paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for path in paths {
            self.traverse_path(path, 0, &mut files)?;
        }
        Ok(files)
    }

    fn traverse_path(&self, path: &Path, depth: u32, files: &mut Vec<PathBuf>) -> Result<()> {
        // 检查深度限制
        if self.max_depth != u32::MAX && depth > self.max_depth {
            return Ok(());
        }

        // 检查是否应该排除
        if self.should_exclude(path) {
            debug!("Excluding: {:?}", path);
            return Ok(());
        }

        // 检查隐藏文件（但对起始路径不进行检查）
        if depth > 0 && !self.include_hidden {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    return Ok(());
                }
            }
        }

        let metadata = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => {
                debug!("Cannot access {:?}: {}", path, e);
                return Ok(());
            }
        };

        // 处理符号链接
        if metadata.file_type().is_symlink() {
            if !self.follow_links {
                return Ok(());
            }
            // 对于符号链接，尝试获取目标元数据
            if let Ok(target_metadata) = fs::metadata(path) {
                if !target_metadata.is_dir() && !target_metadata.is_file() {
                    return Ok(());
                }
            }
        }

        if metadata.is_dir() {
            self.traverse_dir(path, depth, files)?;
        } else if metadata.is_file() {
            // 跳过二进制文件（如果配置了）
            if !self.is_binary_file(path) {
                files.push(path.to_path_buf());
            }
        }

        Ok(())
    }

    fn traverse_dir(&self, dir: &Path, depth: u32, files: &mut Vec<PathBuf>) -> Result<()> {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                debug!("Cannot read directory {:?}: {}", dir, e);
                return Ok(());
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            self.traverse_path(&path, depth + 1, files)?;
        }

        Ok(())
    }

    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.exclude_patterns {
            // 简单的通配符匹配
            if pattern.contains('*') {
                let parts: Vec<&str> = pattern.split('*').collect();
                let mut last = 0;
                let mut matches = true;

                for part in parts {
                    if let Some(pos) = path_str[last..].find(part) {
                        last += pos + part.len();
                    } else {
                        matches = false;
                        break;
                    }
                }

                if matches {
                    return true;
                }
            } else if path_str.contains(pattern.as_str()) {
                return true;
            }
        }

        false
    }

    fn is_binary_file(&self, path: &Path) -> bool {
        match self.binary_mode {
            super::cli::BinaryMode::No => {
                // 检查文件是否包含 null 字节
                if let Ok(content) = fs::read(path) {
                    let check_len = std::cmp::min(8192, content.len());
                    content[..check_len].iter().any(|&b| b == 0)
                } else {
                    false
                }
            }
            super::cli::BinaryMode::Yes => false,
            super::cli::BinaryMode::WithPlaceholder => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_traverse_basic() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        File::create(dir.join("test.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        File::create(dir.join("test.rs"))
            .unwrap()
            .write_all(b"fn main()")
            .unwrap();

        let traverser = Traverser::new(
            u32::MAX,
            vec![],
            false,
            false,
            super::super::cli::BinaryMode::No,
        );
        let files = traverser.traverse(&[dir.to_path_buf()]).unwrap();

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_exclude_pattern() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        File::create(dir.join("test.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        File::create(dir.join("target.txt"))
            .unwrap()
            .write_all(b"world")
            .unwrap();

        let traverser = Traverser::new(
            u32::MAX,
            vec!["target".to_string()],
            false,
            false,
            super::super::cli::BinaryMode::No,
        );
        let files = traverser.traverse(&[dir.to_path_buf()]).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name().unwrap(), "test.txt");
    }
}
