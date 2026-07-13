# Phase 1 — 树遍历解释器（Crafting Interpreters Part II）

> 对应书中 **jlox**：Java + AST 树遍历。本仓库用 **Rust** 实现同等流水线，目标语言为 **JS 子集**。  
> 返回 [路线图总览](README.md)。

## 阶段目标

- 从源码到可执行：`Scanner → Parser → AST → Interpreter`（+ 后期 `Resolver`）。
- 语义正确、可读；**不追求**与原生 JS 引擎同级性能。
- 每节结束后有最小验收；优先扩展 `test/` 下 `.js` 样例 + `cargo test`。

## 进度清单

- [x] Ch.4 Scanning（补齐）
- [x] Ch.5 Representing Code
- [x] Ch.6 Parsing Expressions
- [x] Ch.7 Evaluating Expressions
- [x] Ch.8 Statements and State
- [x] Ch.9 Control Flow
- [ ] Ch.10 Functions
- [ ] Ch.11 Resolving and Binding
- [ ] Ch.12–13 Classes / Inheritance（可选）

---

## Ch.4 Scanning

**书中框架：** Scanner（`source` / `start` / `current` / `line`）、lexeme → token。

**现状：** MVP 已完成。缺口见 [TODO-scanner-tokens.md](../TODO-scanner-tokens.md)。

**本仓库改动：**

- `[src/lib/token.rs](../../src/lib/token.rs)` — 增加比较/逻辑/复合赋值等 `TokenKind`
- `[src/lib/lexer.rs](../../src/lib/lexer.rs)` — 最长匹配多字符运算符；关键字升格
- 建议优先落地：`<` `>` `<=` `>=` `==` `!=` `===` `!==`、`!` `&&` `||`、`%`、`?` `:`

**最小验收：**

```js
var x = 1 + 2 === 3 && !(false);
```

token 流中 `===` / `&&` / `!` 各为一个 token，不被拆碎。

---



## Ch.5 Representing Code

**书中框架：** Context-Free Grammar、AST、**Visitor Pattern**（双分派）。

**本仓库改动：**

- `[src/lib/ast.rs](../../src/lib/ast.rs)` — `Expr` / 后续 `Stmt` 枚举或 struct
- Visitor：Rust 可用 `enum` + `match`，或 `trait Visit` + `accept`；二选一写进模块注释，全项目统一
- 可选 Pretty Printer（调试用）

**最小验收：**

- 手工构造 `Binary(Number(1), Add, Number(2))`，pretty-print 或 `Debug` 输出可读树。

---



## Ch.6 Parsing Expressions

**书中框架：** **Recursive Descent**、文法规则 → 函数、**Panic Mode** 同步到语句边界。

**本仓库改动：**

- `[src/lib/parser.rs](../../src/lib/parser.rs)` — `Parser { tokens, current }`
- 表达式优先级：`primary` → `unary` → `factor` → `term` → `comparison` → `equality` → …
- 错误：`synchronize()` 跳到 `;` / 关键字边界

**最小验收：**

```js
1 + 2 * 3;
-(4);
```

AST 体现 `*` 高于 `+`；一元 `-` 正确挂载。

---



## Ch.7 Evaluating Expressions

**书中框架：** 树遍历求值、运行时值表示、运算符语义与类型错误。

**本仓库改动：**

- 新建 `[src/lib/interpreter.rs](../../src/lib/interpreter.rs)`（**不要**占用 `compiler.rs`）
- 运行时值：`enum Value { Number(f64), String(String), Bool(bool), Null, … }`
- `main`：读文件 → scan → parse → interpret，打印表达式结果或错误

**最小验收：**

```js
1 + 2 * 3;   // → 7
"a" + "b";   // → "ab"（拼接）；Number+Number 相加；其它组合运行时错误（无隐式强制转换）
```

**本仓库语义备注：** `!` 的 truthy 仅 `null`/`false` 为假（`0`、`""` 仍为真，刻意简化）。
---



## Ch.8 Statements and State

**书中框架：** 语句 vs 表达式、全局变量、**Environment** 链、块作用域。

**本仓库改动：**

- `ast.rs` — `Stmt`：`Expression`、`Var`/`Let`/`Const`、`Block`、`Print`（或用 `console.log` 特例）
- 新建 `[src/lib/env.rs](../../src/lib/env.rs)` — `define` / `get` / `assign`，`enclosing: Option`
- Parser：声明与语句入口 `declaration` / `statement`

**最小验收：**

```js
var a = 1;
var b = a + 3;
console.log(b);  // 或约定的 print 原语 → 4
```

与现有 `[test/test.js](../../test/test.js)` 对齐。

---



## Ch.9 Control Flow

**书中框架：** `if` / `while` / `for` 脱糖、逻辑短路（`and`/`or` → `&&`/`||`）。

**本仓库改动：**

- Parser + Interpreter 增加控制流语句
- `for` 可解析为等价 `while` + 初始化/增量（书中做法）

**最小验收：**

```js
var i = 0;
while (i < 3) { i = i + 1; }
if (i === 3) { console.log(i); }
```

---



## Ch.10 Functions

**书中框架：** 函数为值、调用约定、局部 Environment、返回值、`return`。

**本仓库改动：**

- `Stmt::Function`、`Expr::Call`、`Value::Function`
- 调用时新建 Environment，绑定参数；支持闭包雏形（完整正确性在 Ch.11）

**最小验收：**

```js
function add(a, b) { return a + b; }
console.log(add(1, 2));  // 3
```

箭头函数标为延后（见总览语言范围）。

---



## Ch.11 Resolving and Binding

**书中框架：** **Lexical Scope** 精确定义、**Resolver** 语义分析遍、修复闭包捕获时机。

**本仓库改动：**

- 新建 `[src/lib/resolver.rs](../../src/lib/resolver.rs)`
- 在 interpret 前跑 resolve；变量访问带「作用域深度」或等价距离
- 经典用例：声明前调用的闭包不应错误绑定到后声明的同名变量

**最小验收：**

```js
var a = "global";
{
  function showA() { console.log(a); }
  showA();
  var a = "block";
  showA();
}
```

两行输出均应为 `global`（与书中闭包修复目标一致；若采用 `let` TDZ，在文档中注明语义差异）。

---



## Ch.12–13 Classes / Inheritance（可选）

**书中框架：** 类声明、实例、方法、`this`、继承与 `super`。

**本仓库改动：**

- 映射到 JS `class` / `new` / `extends` / `super` 子集
- 可放在 Phase 1 末或与 Phase 2 OOP 章合并；**非 Phase 1 阻塞项**

**最小验收（若做）：**

```js
class Point {
  constructor(x) { this.x = x; }
  getX() { return this.x; }
}
console.log(new Point(1).getX());
```

---



## Phase 1 完成定义

- [ ] `test/test.js` 经 tree-walk 可执行并得到预期输出
- [ ] 表达式、变量、控制流、函数均有单元或集成测试
- [ ] Resolver 覆盖闭包绑定用例
- [ ] `index.rs` 导出新模块；`main` 默认跑 Phase 1 流水线
- [ ] 更新本文件进度清单为 checked
