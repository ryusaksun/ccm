# ccm

Claude Code session Manager — 终端 TUI 工具，用于浏览、搜索和管理本地 Claude Code 会话。

![Rust](https://img.shields.io/badge/Rust-2021-orange)

## 功能

- **会话浏览** — 自动扫描 `~/.claude/projects/` 下的所有会话，展示项目名、首条提示、消息数、分支、时间等
- **实时搜索** — 按提示内容、摘要、项目名、分支名模糊搜索
- **项目过滤** — 按项目筛选会话列表
- **多维排序** — 按修改时间、创建时间、消息数、项目名排序，支持升降序切换
- **会话预览** — 侧边预览面板，实时查看对话内容（User / Assistant / System）
- **恢复会话** — 直接从 TUI 中恢复 Claude Code 会话（`claude -r`），退出后自动返回 TUI
- **批量操作** — 标记多个会话后批量删除
- **导出** — 将会话导出为 Markdown 或 JSON 格式

## 安装

```bash
# 从源码构建
git clone https://github.com/ryusaksun/ccm.git
cd ccm
cargo build --release

# 可执行文件在 target/release/ccm
# 可选：复制到 PATH 中
cp target/release/ccm ~/.local/bin/
```

## 使用

```bash
ccm
```

## 快捷键

| 按键 | 功能 |
|------|------|
| `j` / `k`、`↑` / `↓` | 上下移动 |
| `g` / `G` | 跳到顶部 / 底部 |
| `Ctrl-b` / `Ctrl-f` | 上翻页 / 下翻页 |
| `/` | 搜索 |
| `f` | 按项目过滤 |
| `s` / `S` | 切换排序字段 / 切换排序方向 |
| `Enter`、`r` | 恢复会话 |
| `d` | 删除会话 |
| `m`、`Space` | 标记 / 取消标记 |
| `D` | 删除已标记的会话 |
| `e` | 导出会话 |
| `p` | 切换预览面板 |
| `Tab` | 聚焦 / 取消聚焦预览 |
| `?` | 帮助 |
| `q`、`Esc` | 退出 |

## 依赖

- [ratatui](https://github.com/ratatui/ratatui) — TUI 框架
- [crossterm](https://github.com/crossterm-rs/crossterm) — 跨平台终端控制
- [serde](https://serde.rs/) / serde_json — JSON 解析
- [chrono](https://github.com/chronotope/chrono) — 时间处理

## 许可证

MIT
