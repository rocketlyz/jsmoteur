# Phase 2 — 字节码虚拟机（Crafting Interpreters Part III）

> 对应书中 **clox**：C + 字节码 + 栈式 VM。本仓库用 **Rust** 重做执行后端；语言仍为 Phase 1 的 **JS 子集**。  
> 返回 [路线图总览](README.md)。

## 阶段目标

- 流水线：`源码 →（编译期）Scanner/Parser 或单遍 Compiler → Chunk → VM`
- 相对 Phase 1：**密集字节码 + 栈机**，改善 locality；自管内存（GC）。
- Phase 1 的 `interpreter` **保留**作语义对照与差异测试，不强制删除。

## 进度清单

- [ ] Ch.14 Chunks of Bytecode
- [ ] Ch.15 A Virtual Machine
- [ ] Ch.16 Scanning on Demand
- [ ] Ch.17 Compiling Expressions
- [ ] Ch.18–20 Types / Strings / Hash Tables
- [ ] Ch.21–24 Globals / Locals / Jumping / Calls
- [ ] Ch.25–26 Closures / Garbage Collection
- [ ] Ch.27–30 Classes / Optimization（后期）

---

## Ch.14 Chunks of Bytecode

**书中框架：** 为何不用纯 AST 执行（cache locality）；**Chunk** = 字节码 + 常量池；反汇编。

**本仓库改动：**

- 新建 `src/lib/opcode.rs` — `OpCode` 枚举（可 `repr(u8)`）
- 新建 `src/lib/chunk.rs` — `code: Vec<u8>`、`constants: Vec<Value>`、`lines`
- 手工写入一段 `OP_CONSTANT` / `OP_ADD` / `OP_RETURN`，`disassemble` 打印可读

**最小验收：** 反汇编输出与手写指令一致；常量池可取出数字。

---

## Ch.15 A Virtual Machine

**书中框架：** 栈式 **VM**、指令 dispatch loop、全局 VM 状态（书中简化；嵌入时宜传 `&mut Vm`）。

**本仓库改动：**

- 新建 `src/lib/vm.rs` — `stack`、`ip`、`chunk`；`interpret(chunk) -> Result`
- 实现基础算术与常量加载
- `main` 可选：`--backend=vm` 与 tree-walk 切换（或环境变量）

**最小验收：** 执行手写 chunk：`1 + 2` → 栈顶 `3`。

---

## Ch.16 Scanning on Demand

**书中框架：** 编译期按需取 token，不必先物化整棵 AST（clox 路径）。

**本仓库改动：**

- 复用 / 微调 [`lexer.rs`](../../src/lib/lexer.rs)，供 Compiler 拉流式 token
- 可与 Phase 1 Parser 共用 Scanner；Compiler 不依赖完整 AST（允许「单遍」）

**最小验收：** Compiler 能消费与 Phase 1 相同的 token 流，对简单表达式编译成功。

---

## Ch.17 Compiling Expressions

**书中框架：** **Pratt / Top-Down Operator Precedence** 解析；解析同时发射字节码。

**本仓库改动：**

- [`src/lib/compiler.rs`](../../src/lib/compiler.rs) — Pratt 表（`prefix` / `infix` / precedence）
- 输出 `Chunk`；错误报告带行号

**最小验收：**

```js
1 + 2 * 3
```

编译后 VM 执行结果为 `7`，与 Phase 1 解释器一致。

---

## Ch.18–20 Types / Strings / Hash Tables

**书中框架：** 动态 **Value** 标签（或 NaN boxing）、堆对象、字符串驻留、**开放寻址哈希表**。

**本仓库改动：**

- 新建 `src/lib/value.rs` — 与 Phase 1 `Value` 对齐或统一为一套
- 新建 `src/lib/table.rs` — 全局变量名、字符串 intern 等
- 字符串运算、真值规则与 Phase 1 文档化一致

**最小验收：**

```js
var s = "hi";
console.log(s);
```

VM 路径与 tree-walk 输出一致；表查找全局 `s` 成功。

---

## Ch.21–24 Globals / Locals / Jumping / Calls

**书中框架：** 全局名；局部栈槽；**backpatching** 跳转；调用帧 / 函数对象。

**本仓库改动：**

- Compiler：作用域、局部变量槽位、`OP_JUMP` / `OP_JUMP_IF_FALSE` 回填
- VM：`CallFrame`、`OP_CALL` / `OP_RETURN`
- 控制流与函数语义对齐 Phase 1

**最小验收：**

- [`test/test.js`](../../test/test.js) 在 VM 后端跑通
- `if` / `while` / 简单 `function` 有对照测试（tree-walk vs VM）

---

## Ch.25–26 Closures / Garbage Collection

**书中框架：** **Upvalue**（Lua 风格间接层）；**Mark-and-Sweep**；roots = 栈 + 全局 + 编译器/VM 临时引用。

**本仓库改动：**

- Compiler：`resolveUpvalue` / `OP_GET_UPVALUE` / `OP_CLOSE_UPVALUE`
- 新建 `src/lib/gc.rs` — mark（从 roots 追踪）+ sweep；与 `Value` 堆对象协作
- Rust：可用自管堆索引 / arena；避免与标准 `Rc` 语义 silently 混用而不文档化

**最小验收：**

- 闭包捕获外层局部，外层返回后仍可读
- 分配足够临时对象后 GC 回收不可达对象（可用调试计数断言）

---

## Ch.27–30 Classes… / Optimization（后期）

**书中框架：** 类与实例、方法、`super`；benchmark、profiler、哈希探测等微优化。

**本仓库改动：**

- OOP：与 Phase 1 可选 class 章对齐后再上 VM 对象模型
- 优化：建立小型 `test/bench/`；改动前后计时；避免过拟合微基准

**最小验收（若做）：** 选定 1–2 个脚本，VM 明显快于 tree-walk（数量级或稳定倍数，记录在注释/文档）。

---

## Phase 2 完成定义

- [ ] 同一 JS 子集：tree-walk 与 VM 结果一致（核心测试套）
- [ ] Chunk 可反汇编；VM 可跑 `test/test.js`
- [ ] 闭包 + GC 有基础测试
- [ ] `compiler.rs` 仅服务字节码；`interpreter.rs` 仍可独立运行
- [ ] 更新本文件进度清单为 checked

## 与 Phase 1 的关系

| 项目 | Phase 1 | Phase 2 |
|------|---------|---------|
| 前端 | Scanner + Recursive Descent → AST | Scanner + Pratt（可单遍）→ Chunk |
| 后端 | Interpreter 树遍历 | Stack VM |
| 内存 | 主要依赖 Rust/宿主 | 自研 GC（+ Rust 分配器） |
| 目标 | 正确、可理解 | 正确 + 更快 |
