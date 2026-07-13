# Jsmoteur
> Jsmoteur is a javascript engine written in rust.

## Files
+ ast.rs: 语法树（AST）
+ lexer.rs: 词法分析（Scanner）
+ parser.rs: 语法分析（Parser）
+ token.rs: Token 定义
+ symbol.rs: 关键字与运算符表
+ interpreter.rs / env.rs / resolver.rs: Phase 1 树遍历（规划中，见路线图）
+ compiler.rs / vm.rs / …: Phase 2 字节码 VM（规划中，见路线图）

## 语法
### 1. 变量
+ let 
+ const
### 2. 表达式
+ 等值：== 
+ 算术表达： \+ \-  * /
+ 位运算： 
### 3. 语句
+ 条件语句： if else、switch、do while
### 4. 函数
+ 普通函数
+ 箭头函数


## 实现

开发顺序对照《Crafting Interpreters》Part II / Part III，详见：

- **[docs/roadmap/README.md](docs/roadmap/README.md)** — 总路线图与章节对照
- [docs/roadmap/phase-1-treewalk.md](docs/roadmap/phase-1-treewalk.md) — 树遍历解释器
- [docs/roadmap/phase-2-bytecode-vm.md](docs/roadmap/phase-2-bytecode-vm.md) — 字节码 VM
- [docs/TODO-scanner-tokens.md](docs/TODO-scanner-tokens.md) — Scanner MVP 未实现 token

### 词法解析

Phase 0 Scanner MVP 已完成（`cargo run -- test/test.js`）。多字符运算符等见 TODO 文档。
