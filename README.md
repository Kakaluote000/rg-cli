# rg-cli - Rust CLI 文件搜索工具

一个用 Rust 开发的、类似 ripgrep 的快速文件搜索工具。

## 当前状态

### ✅ Day 1 完成（2026-03-29）

- [x] 设计命令行参数（搜索路径、模式、排除规则）
- [x] 创建项目结构（使用 cargo new）
- [x] 集成 clap 库（参数解析）
- [x] 实现文件遍历逻辑（递归遍历目录）
- [x] 集成正则表达式（regex crate）
- [x] 实现搜索匹配逻辑
- [x] 添加并发搜索（rayon crate）
- [x] 实现颜色高亮输出
- [x] 添加性能统计（时间、文件数）
- [x] 编写单元测试

GitHub 仓库已发布：https://github.com/Kakaluote000/rg-cli

## 项目结构

```
src/
├── main.rs          # 主入口
├── cli/             # 命令行参数解析
├── error/           # 错误类型定义
├── highlight/        # 颜色高亮输出
├── search/          # 搜索匹配逻辑
├── stats/           # 性能统计
└── traversal/       # 文件遍历
```

## 使用示例

```bash
# 在当前目录搜索 "fn"
cargo run -- "fn"

# 指定搜索路径
cargo run -- "SearchEngine" src/

# 忽略大小写
cargo run -- -i "HELLO" src/

# 只显示文件名
cargo run -- -l "fn" src/

# 排除特定目录
cargo run -- -e "target" "fn" .
```

## 功能特性

- ✅ 正则表达式搜索
- ✅ 大小写不敏感模式
- ✅ 并发搜索
- ✅ 颜色高亮输出
- ✅ 性能统计
- ✅ 排除模式支持
- ✅ 深度控制
- ✅ 符号链接跟随
- ✅ 隐藏文件支持

## 依赖项

- clap - 命令行参数解析
- regex - 正则表达式
- rayon - 并发处理
- anyhow - 错误处理
- tracing - 日志追踪

## 开发进度

### Week 1: 项目搭建 + 核心功能 ✅ 完成
- Day 1-3: 项目搭建、CLI 参数、项目结构
- Day 4-6: 文件遍历、正则表达式、搜索匹配

### Week 2: 高级功能
- Day 7-9: 并发搜索、颜色高亮、性能统计（Day 1 已完成）
- Day 10-12: 性能优化、错误处理完善、单元测试（Day 1 已完成）

### Week 3: 发布准备
- Day 13-15: README 完善、使用文档、GitHub 发布（Day 1 已完成）

## License

MIT
