//! 性能统计模块
//!
//! 收集和显示搜索性能统计信息

use crate::error::SearchResult;
use std::fmt;
use std::time::{Duration, Instant};

/// 搜索统计信息收集器
pub struct SearchStats {
    start_time: Instant,
    files_searched: usize,
    files_matched: usize,
    bytes_searched: u64,
}

impl SearchStats {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            files_searched: 0,
            files_matched: 0,
            bytes_searched: 0,
        }
    }

    /// 更新统计信息
    pub fn update(&mut self, results: &[SearchResult]) {
        self.files_matched = results
            .iter()
            .map(|r| r.path.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len();
    }

    /// 完成统计并生成报告
    pub fn finish(self, results: &[SearchResult], elapsed: Duration) -> SearchReport {
        SearchReport {
            total_matches: results.len(),
            files_with_matches: results
                .iter()
                .map(|r| r.path.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            files_searched: self.files_searched,
            bytes_searched: self.bytes_searched,
            elapsed,
        }
    }
}

impl Default for SearchStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 搜索报告
#[derive(Debug, Clone)]
pub struct SearchReport {
    /// 总匹配数
    pub total_matches: usize,
    /// 匹配的文件数
    pub files_with_matches: usize,
    /// 搜索的文件数
    pub files_searched: usize,
    /// 搜索的字节数
    pub bytes_searched: u64,
    /// 耗时
    pub elapsed: Duration,
}

impl fmt::Display for SearchReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(
            f,
            "Searched {} files, {} files with matches",
            self.files_searched, self.files_with_matches
        )?;
        writeln!(f, "Total matches: {}", self.total_matches)?;

        if self.bytes_searched > 0 {
            let speed = self.bytes_searched as f64 / self.elapsed.as_secs_f64();
            let speed_str = if speed > 1_000_000.0 {
                format!("{:.1} MB/s", speed / 1_000_000.0)
            } else if speed > 1_000.0 {
                format!("{:.1} KB/s", speed / 1_000.0)
            } else {
                format!("{:.0} B/s", speed)
            };
            writeln!(f, "Speed: {}", speed_str)?;
        }

        writeln!(f, "Time: {:?}", self.elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_format() {
        let report = SearchReport {
            total_matches: 42,
            files_with_matches: 3,
            files_searched: 10,
            bytes_searched: 1_000_000,
            elapsed: Duration::from_millis(150),
        };

        let output = format!("{}", report);
        assert!(output.contains("42"));
        assert!(output.contains("3 files with matches"));
    }
}
