# Scanner MVP — 未实现 Token TODO

> 对应 Crafting Interpreters 风格 Scanning 的第一版边界。  
> 已实现见 [`src/lib/token.rs`](src/lib/token.rs) / [`src/lib/lexer.rs`](src/lib/lexer.rs)。  
> 完整运算符表见 [`src/lib/symbol.rs`](src/lib/symbol.rs) 的 `Denotation`。

## 已实现（MVP）

| 类别 | Token |
|------|--------|
| 标点 | `( ) { } [ ] , . ;` |
| 单字符运算 | `=` `+` `-` `*` `/` |
| 字面量 | `Identifier`、`Number`、`"string"`（含 `\"` `\\`）、`true`/`false`/`null` |
| 关键字（升格为 TokenKind） | `var` `let` `const` `function` `if` `else` `return` `while` `for` `class` `new` `this` `super` |
| 其它 | `Eof`、`Error`；`//` 行注释、`/* */` 块注释（不嵌套） |

---

## 1. 多字符运算符（下一优先）

`Denotation` 已定义，Scanner 尚未识别；当前会被拆成多个单字符 token 或 Error。

### 比较 / 相等

| Lexeme | Denotation | TokenKind（待加） |
|--------|------------|-------------------|
| `<` | `LT` | `LT` |
| `>` | `GT` | `GT` |
| `<=` | `LE` | `LE` |
| `>=` | `GE` | `GE` |
| `==` | `Eq` | `Eq` |
| `!=` | `NotEq` | `NotEq` |
| `===` | `EqStrict` | `EqStrict` |
| `!==` | `NotEqStrict` | `NotEqStrict` |

### 算术 / 一元

| Lexeme | Denotation | TokenKind（待加） |
|--------|------------|-------------------|
| `%` | `Mod` | `Mod` |
| `++` | `Inc` | `Inc` |
| `--` | `Dec` | `Dec` |
| `!` | `Not` | `Not` |
| `~` | `BitNot` | `BitNot` |

### 位运算 / 移位

| Lexeme | Denotation | TokenKind（待加） |
|--------|------------|-------------------|
| `&` | `BitAnd` | `BitAnd` |
| `|` | `BitOr` | `BitOr` |
| `^` | `BitXor` | `BitXor` |
| `<<` | `SHL` | `SHL` |
| `>>` | `SAR` | `SAR` |
| `>>>` | `SHR` | `SHR` |

### 逻辑 / 三元 / 冒号

| Lexeme | Denotation | TokenKind（待加） |
|--------|------------|-------------------|
| `&&` | `And` | `And` |
| `||` | `Or` | `Or` |
| `?` | `Conditional` | `Conditional` |
| `:` | `Colon` | `Colon` |

### 复合赋值

| Lexeme | Denotation | TokenKind（待加） |
|--------|------------|-------------------|
| `+=` | `AssignAdd` | `AssignAdd` |
| `-=` | `AssignSub` | `AssignSub` |
| `*=` | `AssignMul` | `AssignMul` |
| `/=` | `AssignDiv` | `AssignDiv` |
| `%=` | `AssignMod` | `AssignMod` |
| `<<=` | `AssignSHL` | `AssignSHL` |
| `>>=` | `AssignSAR` | `AssignSAR` |
| `>>>=` | `AssignSHR` | `AssignSHR` |
| `&=` | `AssignBitAnd` | `AssignBitAnd` |
| `|=` | `AssignBitOr` | `AssignBitOr` |
| `^=` | `AssignBitXor` | `AssignBitXor` |

**实现提示：** 在 `scan_token` 里对 `=` `+` `-` `<` `>` `!` `&` `|` 等做 `match_char` / `peek` 最长匹配（如 `===` 优先于 `==` 优先于 `=`）。

---

## 2. 关键字：已识别但未升格为 TokenKind

`keyword_from_str` 能认出，但 Scanner 仍发成 `Identifier`：

| Keyword | Lexeme |
|---------|--------|
| `Break` | `break` |
| `Case` | `case` |
| `Catch` | `catch` |
| `Continue` | `continue` |
| `Default` | `default` |
| `Delete` | `delete` |
| `Do` | `do` |
| `Enum` | `enum` |
| `Export` | `export` |
| `Extends` | `extends` |
| `Finally` | `finally` |
| `Import` | `import` |
| `In` | `in` |
| `Instanceof` | `instanceof` |
| `Switch` | `switch` |
| `Throw` | `throw` |
| `Try` | `try` |
| `Typeof` | `typeof` |
| `Void` | `void` |
| `With` | `with` |

**实现提示：** 在 `identifier()` 的 `Keyword` match 中为上述项增加 `TokenKind` 变体并映射。

---

## 3. 字面量 / 词法扩展（延后）

| 项 | 说明 | 难点 |
|----|------|------|
| 正则字面量 `/.../` | 与除法 `/` 同形 | 需 Parser 上下文，非纯 Scanner |
| 模板字符串 `` `...${}` `` | ES6 | 需插值分段 / 多 token 状态 |
| 单引号字符串 `'...'` | MVP 仅 `"` | 与双引号对称即可 |
| 数字扩展 | `0x`/`0b`/`0o`、科学计数 `1e3`、BigInt `1n` | 词法规则加长 |
| Unicode 标识符 | `\uXXXX`、非 ASCII 字母 | 需 Unicode ID_Start/Continue |
| 嵌套块注释 | `/* /* */ */` | 当前不支持嵌套 |

---

## 4. 建议落地顺序

1. **比较与相等**（`<` `>` `<=` `>=` `==` `!=` `===` `!==`）— 表达式解析刚需  
2. **逻辑与一元**（`!` `&&` `||` `%` `++` `--` `?` `:`）  
3. **复合赋值与位运算**  
4. **剩余关键字升格**（按 Parser 需要：`break`/`continue`/`switch`/`try`/`catch` 等）  
5. **单引号字符串 + 数字扩展**  
6. **模板字符串 / 正则**（需 Parser 协同）

---

## 5. 验收清单（完成后勾选）

- [ ] `TokenKind` 覆盖 `Denotation` 全集  
- [ ] Scanner 最长匹配多字符运算符  
- [ ] 全部 `Keyword` 映射到独立 `TokenKind`（或明确保留为 Identifier 的策略文档）  
- [ ] 单引号字符串测试  
- [ ] `==` / `===` / `+=` / `>>>` 等单元测试  
- [ ] 更新本文件：将已完成项移到「已实现」
