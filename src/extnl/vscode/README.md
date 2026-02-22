# Volki VS Code Extension

Syntax highlighting for `.volki` files in Visual Studio Code. Supports Rust code with RSX (JSX/HTML-like) templating.

## Features

- **Syntax highlighting** for Rust keywords, types, operators, strings, numbers, and comments
- **RSX tag highlighting** for HTML element names, component names, attributes, and tag brackets
- **Nested block comments** (`/* /* inner */ outer */`)
- **Raw string support** (`r#"..."#`)
- **Auto-closing pairs** for braces, brackets, parens, angle brackets, and quotes
- **Bracket matching** for `{}`, `[]`, `()`, `<>`
- **Code folding** via `#region` / `#endregion` markers

## Installation

### From VSIX

```bash
code --install-extension volki-language-support-0.1.0.vsix
```

### From Source (Development)

1. Copy or symlink `src/extnl/vscode/` into your VS Code extensions directory
2. Reload VS Code
3. Open any `.volki` file — syntax highlighting activates automatically

### Development with Extension Host

1. Open `src/extnl/vscode/` as a folder in VS Code
2. Press **F5** to launch the Extension Development Host
3. In the new window, create or open a `.volki` file
4. Verify syntax highlighting is applied

## Supported Tokens

| Token | Scope | Example |
|-------|-------|---------|
| Keyword (control) | `keyword.control.volki` | `if`, `else`, `for`, `while`, `match`, `return` |
| Keyword (other) | `keyword.other.volki` | `pub`, `fn`, `let`, `mut`, `struct`, `impl` |
| Type | `entity.name.type.volki` | `Html`, `String`, `Vec`, `Option`, `i32`, `bool` |
| Boolean | `constant.language.volki` | `true`, `false` |
| Tag name | `entity.name.tag.volki` | `div`, `span`, `h1`, `input`, `nav` |
| Component | `entity.name.tag.component.volki` | `Button`, `NavBar`, `AppLayout` |
| Tag bracket | `punctuation.definition.tag.*.volki` | `<`, `</`, `>`, `/>` |
| Attribute | `entity.other.attribute-name.volki` | `class`, `onclick`, `href`, `style` |
| String | `string.quoted.double.volki` | `"hello"` |
| Raw string | `string.quoted.double.raw.volki` | `r#"raw string"#` |
| Number | `constant.numeric.*.volki` | `42`, `3.14`, `0xff`, `0b1010` |
| Line comment | `comment.line.double-slash.volki` | `// comment` |
| Doc comment | `comment.line.documentation.volki` | `//! doc comment` |
| Block comment | `comment.block.volki` | `/* comment */` |
| Operator | `keyword.operator.*.volki` | `->`, `=>`, `::`, `==`, `&&` |
| Embedded expr | `meta.embedded.expression.volki` | `{variable}` in attributes |

## RSX Tag Recognition

The grammar recognizes the same HTML/RSX tags as the JetBrains plugin:

**Structure:** `div`, `span`, `p`, `a`, `header`, `footer`, `main`, `section`, `article`, `aside`, `nav`, `body`, `html`

**Headings:** `h1`, `h2`, `h3`, `h4`, `h5`, `h6`

**Table:** `table`, `tr`, `td`, `th`, `thead`, `tbody`, `tfoot`

**Form:** `input`, `button`, `form`, `textarea`, `select`, `option`, `label`

**List:** `ul`, `ol`, `li`, `dl`, `dt`, `dd`

**Text:** `strong`, `em`, `b`, `i`, `u`, `small`, `sub`, `sup`, `pre`, `code`, `blockquote`

**Media:** `img`, `br`, `hr`, `video`, `audio`, `source`, `canvas`, `svg`, `path`

**Meta/Semantic:** `meta`, `link`, `script`, `style`, `title`, `head`, `details`, `summary`, `dialog`, `figure`, `figcaption`

Any tag starting with an uppercase letter (e.g., `<Button>`, `<NavBar>`) is treated as a **component**.

## How RSX Detection Works

The grammar distinguishes RSX tags from comparison operators using the same heuristic as the JetBrains plugin:

- `<div ...>` is an RSX tag (recognized tag name after `<`)
- `<Button ...>` is an RSX component (uppercase letter after `<`)
- `x < 5` is a comparison operator (no recognized tag or uppercase after `<`)
