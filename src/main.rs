//! rg-cli - A fast, cross-platform file search tool similar to ripgrep
//!
//! ## 项目结构
//!
//! - `cli` - 命令行参数解析 (clap)
//! - `search` - 搜索匹配逻辑
//! - `traversal` - 文件遍历
//! - `highlight` - 颜色高亮
//! - `stats` - 性能统计
//! - `error` - 错误类型定义

mod cli;
mod error;
mod highlight;
mod search;
mod stats;
mod traversal;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::cli::Cli;
use crate::search::SearchEngine;
use crate::stats::SearchStats;

fn init_tracing(verbose: bool) {
    let filter = if verbose {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}

fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let start_time = std::time::Instant::now();
    let stats = SearchStats::new();

    // 构建搜索路径列表
    let paths: Vec<PathBuf> = if cli.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        cli.paths.clone()
    };

    tracing::info!("Starting search with {} paths", paths.len());
    tracing::info!("Pattern: {}", cli.pattern);
    tracing::info!("Case sensitive: {}", !cli.ignore_case);

    let engine = SearchEngine::new(
        &cli.pattern,
        !cli.ignore_case,
        cli.color(),
        cli.max_count,
        cli.max_depth,
        cli.follow,
        cli.binary,
        &cli.exclude,
    )?;

    let results = engine.search(&paths, cli.hidden)?;

    // 输出结果
    highlight::print_results(&results, cli.color())?;

    // 输出统计信息
    let elapsed = start_time.elapsed();
    let report = stats.finish(&results, elapsed);
    if !cli.no_stats {
        eprintln!("{}", report);
    }

    // 设置退出码：如果有匹配结果则退出码为 0，否则为 1
    if results.is_empty() {
        process::exit(1);
    }

    Ok(())
}
