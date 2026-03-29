# Rust 实战项目 - CLI 文件搜索工具（Day 1 完成）

## 项目概述

今天完成了 Rust CLI 文件搜索工具 "rg-cli" 的开发，这是一个类似 ripgrep 的快速文件搜索工具。

## 技术实现

### 核心技术栈

- **clap 4**: 命令行参数解析，使用 derive 特性
- **regex 1**: 正则表达式搜索
- **rayon 1**: 并发搜索，利用多核 CPU
- **anyhow 1**: 简化的错误处理
- **tracing 0.1**: 结构化日志追踪

### 项目架构

项目采用模块化设计，主要模块包括：

1. **cli/mod.rs**: 使用 clap derive 模式定义 CLI 参数，支持 20+ 个选项
2. **traversal/mod.rs**: 递归文件遍历，支持排除模式、深度控制、符号链接跟随
3. **search/mod.rs**: 基于 regex 的搜索引擎，支持并发搜索
4. **highlight/mod.rs**: ANSI 颜色高亮输出，支持终端检测
5. **error/mod.rs**: 错误类型定义
6. **stats/mod.rs**: 性能统计（匹配数、文件数、耗时、速度）

### 核心功能实现

#### 1. 命令行参数解析

使用 clap 的 derive 特性，简洁地定义了所有 CLI 参数：

```rust
#[derive(Parser, Debug)]
pub struct Cli {
    /// 搜索模式（正则表达式）
    pub pattern: String,
    /// 搜索路径
    pub paths: Vec<PathBuf>,
    /// 忽略大小写
    #[arg(short, long)]
    pub ignore_case: bool,
    // ... 更多参数
}
```

支持的参数包括：搜索模式、路径、大小写控制、深度限制、排除模式、文件匹配模式、颜色输出等。

#### 2. 文件遍历

递归遍历目录，支持：
- 深度控制（`max_depth`）
- 排除模式（支持通配符 `*`）
- 符号链接跟随
- 隐藏文件过滤
- 二进制文件检测（通过 null 字节检测）

关键实现细节：
```rust
fn traverse_path(&self, path: &Path, depth: u32, files: &mut Vec<PathBuf>) {
    // 深度检查
    if depth > self.max_depth { return; }
    // 排除模式检查
    if self.should_exclude(path) { return; }
    // 隐藏文件检查（但对起始路径不检查）
    if depth > 0 && !self.include_hidden && path.hidden() { return; }
    // 递归遍历
}
```

#### 3. 并发搜索

使用 rayon 实现并发搜索，自动利用多核 CPU：

```rust
pub fn search(&self, paths: &[PathBuf]) -> Result<Vec<SearchResult>> {
    let files = self.traverser.traverse(paths)?;
    // 使用 par_iter 并发搜索多个文件
    let results = files
        .par_iter()
        .flat_map(|file| self.search_file(file))
        .collect();
    Ok(results)
}
```

每个文件的搜索仍然串行进行，但多个文件可以并发处理。

#### 4. 颜色高亮输出

ANSI 转义码实现终端颜色高亮：

- 文件名：蓝色（`\x1b[34m`）
- 行号：灰色（`\x1b[38;5;245m`）
- 匹配文本：反显（`\x1b[7m`）

自动检测终端环境，仅在连接 TTY 时启用颜色。

#### 5. 性能统计

收集并显示搜索统计信息：

```rust
struct SearchReport {
    total_matches: usize,
    files_with_matches: usize,
    files_searched: usize,
    bytes_searched: u64,
    elapsed: Duration,
}
```

输出示例：
```
Searched 0 files, 7 files with matches
Total matches: 36
Time: 16.517958ms
```

### 测试覆盖

编写了 6 个单元测试，覆盖：
- 基本文件遍历
- 排除模式
- 基本搜索
- 正则表达式搜索
- 大小写不敏感

测试结果：`6 passed; 0 failed`

## 遇到的问题与解决

### 问题 1: Clap 短选项冲突

多个参数使用了相同的短选项（如 `-f` 被 `fixed_strings` 和 `files_with_matches` 同时使用，`-f` 又被 `follow` 使用）。

**解决**：
- 移除 `fixed_strings`（暂未实现）
- `files_with_matches` 使用 `-l`（与 ripgrep 一致）
- `files_without_matches` 使用 `-L`
- `line_numbers` 使用 `-n`
- `follow` 只使用长选项 `--follow`

### 问题 2: 隐藏文件测试失败

测试创建的临时目录 `.tmpXXX` 被识别为隐藏文件而被排除，导致测试失败。

**解决**：
修改 `traverse_path` 函数，只在 `depth > 0` 时检查隐藏文件：
```rust
if depth > 0 && !self.include_hidden {
    if path.hidden() { return; }
}
```

这样可以搜索隐藏目录内的非隐藏文件。

### 问题 3: 终端检测

使用 `std::io::IsTerminal::is_terminal(&std::io::stdout())` 检测是否连接终端，同时支持 `FORCE_COLOR` 环境变量覆盖。

## 性能表现

在项目中搜索 "struct" 关键字：
- 搜索 5 个源文件
- 找到 7 个匹配
- 耗时约 10-20ms（调试模式）

使用 release 模式编译后性能更好：
```bash
cargo run --release -- "pattern" path/
```

## 项目状态

### ✅ 已完成功能

- Week 1 Day 1-3: 项目搭建、CLI 参数、项目结构
- Week 1 Day 4-6: 文件遍历、正则表达式、搜索匹配
- Week 2 Day 7-9: 并发搜索、颜色高亮、性能统计
- Week 2 Day 10-12: 性能优化、错误处理完善、单元测试
- Week 3 Day 13-15: README 完善、GitHub 发布

### 📝 下一步计划

虽然项目已完成基本功能，但还有很多优化空间：

1. **性能优化**：
   - 使用 `ignore` crate 优化文件遍历
   - 实现内存映射（mmap）读取大文件
   - 优化正则表达式编译和缓存

2. **功能增强**：
   - 实现 `--fixed-strings` 字面字符串匹配
   - 支持 `-C`, `-B`, `-A` 上下文显示
   - 支持多个搜索模式（AND/OR 逻辑）
   - 支持 `-v` 反向匹配（不包含模式的行）

3. **用户体验**：
   - 支持配置文件（`.rgignore`, `.config/rg/config`）
   - 更丰富的输出格式（JSON, grep 兼容等）
   - 更好的错误提示和帮助信息

4. **代码质量**：
   - 增加更多单元测试
   - 添加集成测试
   - 代码覆盖率检查

## 学习收获

1. **Rust 模块系统**：深入理解了 Rust 的模块系统、可见性、模块路径
2. **错误处理**：anyhow 和 thiserror 的区别和适用场景
3. **并发编程**：rayon 的使用，数据并行（data parallelism）
4. **CLI 开发**：clap 的 derive 特性，命令行工具最佳实践
5. **终端编程**：ANSI 转义码，终端检测，颜色输出
6. **Git 与 GitHub**：使用 gh CLI 创建仓库，SSH 推送

## 总结

Day 1 就完成了项目的全部核心功能，展示了 Rust 在 CLI 工具开发方面的优势：

- 类型安全
- 零成本抽象
- 优秀的并发支持
- 丰富的生态系统（clap, rayon, regex）

项目已发布到 GitHub：https://github.com/Kakaluote000/rg-cli

代码质量优先，性能可优化，保持简洁和可读性。
