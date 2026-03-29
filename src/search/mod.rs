//! 搜索匹配模块
//!
//! 负责正则表达式匹配、并发搜索

use crate::cli::BinaryMode;
use crate::error::SearchResult;
use crate::traversal::Traverser;
use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

/// 搜索引擎配置
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// 正则表达式
    pub regex: Regex,
    /// 是否大小写敏感
    pub case_sensitive: bool,
    /// 是否启用颜色
    pub use_color: bool,
    /// 每个文件最大匹配数
    pub max_count: u32,
    /// 排除模式
    pub exclude_patterns: Vec<String>,
    /// 二进制模式
    pub binary_mode: BinaryMode,
}

impl SearchConfig {
    pub fn new(
        pattern: &str,
        case_sensitive: bool,
        use_color: bool,
        max_count: u32,
        exclude_patterns: &[String],
        binary_mode: BinaryMode,
    ) -> Result<Self> {
        let regex = if case_sensitive {
            Regex::new(pattern)
        } else {
            Regex::new(&format!("(?i){}", pattern))
        }?;

        Ok(Self {
            regex,
            case_sensitive,
            use_color,
            max_count,
            exclude_patterns: exclude_patterns.to_vec(),
            binary_mode,
        })
    }
}

/// 搜索引擎
pub struct SearchEngine {
    config: Arc<SearchConfig>,
    traverser: Arc<Traverser>,
}

impl SearchEngine {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pattern: &str,
        case_sensitive: bool,
        use_color: bool,
        max_count: u32,
        max_depth: u32,
        follow_links: bool,
        binary_mode: BinaryMode,
        exclude_patterns: &[String],
    ) -> Result<Self> {
        let config = Arc::new(SearchConfig::new(
            pattern,
            case_sensitive,
            use_color,
            max_count,
            exclude_patterns,
            binary_mode.clone(),
        )?);

        let traverser = Arc::new(Traverser::new(
            max_depth,
            exclude_patterns.to_vec(),
            follow_links,
            false, // include_hidden - passed at search time
            binary_mode,
        ));

        Ok(Self { config, traverser })
    }

    /// 搜索多个路径
    pub fn search(&self, paths: &[PathBuf], include_hidden: bool) -> Result<Vec<SearchResult>> {
        info!("Starting search in {:?}", paths);

        // 获取文件列表
        let files = self.traverser.traverse(paths)?;
        info!("Found {} files to search", files.len());

        // 并发搜索
        let results = files
            .par_iter()
            .flat_map(|file| {
                self.search_file(file, include_hidden)
                    .unwrap_or_default()
            })
            .collect();

        Ok(results)
    }

    /// 搜索单个文件
    fn search_file(&self, path: &PathBuf, include_hidden: bool) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // 检查隐藏文件
        if !include_hidden {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    return Ok(results);
                }
            }
        }

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut line_number = 0u32;
        let mut match_count = 0u32;

        for line in reader.lines().flatten() {
            line_number += 1;

            // 检查是否达到最大匹配数
            if match_count >= self.config.max_count {
                break;
            }

            // 查找所有匹配
            let matches: Vec<_> = self.config.regex.find_iter(&line).collect();

            if !matches.is_empty() {
                for m in matches {
                    if match_count >= self.config.max_count {
                        break;
                    }
                    results.push(SearchResult::new(
                        path.to_string_lossy().to_string(),
                        Some(line_number),
                        line.clone(),
                        m.start(),
                        m.end(),
                    ));
                    match_count += 1;
                }
            }

            // 防止单个文件结果过多
            if results.len() > 10000 {
                debug!("Too many matches in {:?}, truncating", path);
                break;
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_search_basic() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        File::create(dir.join("test.txt"))
            .unwrap()
            .write_all(b"hello world\nrust is awesome\nhello again")
            .unwrap();

        let engine = SearchEngine::new(
            "hello",
            true,
            false,
            u32::MAX,
            u32::MAX,
            false,
            BinaryMode::No,
            &[],
        )
        .unwrap();
        let results = engine.search(&[dir.to_path_buf()], false).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line, "hello world");
        assert_eq!(results[1].line, "hello again");
    }

    #[test]
    fn test_search_regex() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        File::create(dir.join("test.txt"))
            .unwrap()
            .write_all(b"test123\ntest456\nother789")
            .unwrap();

        let engine = SearchEngine::new(
            r"test\d+",
            true,
            false,
            u32::MAX,
            u32::MAX,
            false,
            BinaryMode::No,
            &[],
        )
        .unwrap();
        let results = engine.search(&[dir.to_path_buf()], false).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_case_insensitive() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        File::create(dir.join("test.txt"))
            .unwrap()
            .write_all(b"Hello HELLO hello")
            .unwrap();

        let engine = SearchEngine::new(
            "hello",
            false,
            false,
            u32::MAX,
            u32::MAX,
            false,
            BinaryMode::No,
            &[],
        )
        .unwrap();
        let results = engine.search(&[dir.to_path_buf()], false).unwrap();

        assert_eq!(results.len(), 3);
    }
}
