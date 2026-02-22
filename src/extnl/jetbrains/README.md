# Volki JetBrains Plugin

Syntax highlighting, brace matching, and commenter support for `.volki` files in JetBrains IDEs (IntelliJ IDEA, RustRover, CLion, WebStorm, etc.).

## Features

- **Syntax highlighting** for Rust code + RSX (JSX/HTML-like) templating
- **Brace matching** for `{}`, `()`, `[]`
- **Commenter** support (`//` line comments, `/* */` block comments)
- **Color settings page** for customizing token colors
- Recognizes Rust keywords, types, operators, strings, numbers, and comments
- Highlights HTML tag names, attributes, and tag brackets in RSX blocks
- Detects RSX context by recognizing `<tagname` and `<Component` patterns

## Building

Requires JDK 17+ and Gradle.

```bash
cd src/extnl/jetbrains
./gradlew buildPlugin
```

The built plugin zip will be in `build/distributions/`.

## Installation

1. Build the plugin (see above)
2. In your JetBrains IDE, go to **Settings > Plugins > Gear icon > Install Plugin from Disk...**
3. Select the `.zip` file from `build/distributions/`
4. Restart the IDE

## Development

To run a sandboxed IDE instance with the plugin loaded:

```bash
./gradlew runIde
```

## Supported Token Types

| Token | Example |
|-------|---------|
| Keyword | `pub`, `fn`, `let`, `mut`, `struct`, `impl` |
| Type | `Html`, `String`, `Vec`, `Option`, `i32` |
| Tag name | `div`, `span`, `h1`, `Button` (uppercase = component) |
| Tag bracket | `<`, `</`, `>`, `/>` |
| Attribute | `class`, `onclick`, `href` |
| String | `"hello"`, `r#"raw"#` |
| Number | `42`, `3.14`, `0xff` |
| Comment | `//`, `//!`, `/* */` |

## RSX Highlighting

The plugin provides full syntax highlighting for RSX — an HTML/JSX-like templating syntax embedded in Rust code. The lexer detects RSX context automatically by analyzing what follows a `<` character.

### RSX Detection Heuristic

Not every `<` is a tag — the lexer distinguishes RSX from operators:

- **`<div ...>`** — recognized as an RSX tag because `div` is in the known tag list
- **`<Button ...>`** — recognized as an RSX component because it starts with an uppercase letter
- **`x < 5`** — treated as a comparison operator because `x` is not a known tag and doesn't start uppercase

This means `<` followed by a known HTML tag name or an uppercase identifier enters RSX mode, while all other uses remain as operators.

### Supported HTML/RSX Tag Names

The plugin recognizes the following HTML tags for RSX highlighting:

**Structure:** `div`, `span`, `p`, `a`, `body`, `html`

**Headings:** `h1`, `h2`, `h3`, `h4`, `h5`, `h6`

**Semantic:** `header`, `footer`, `main`, `section`, `article`, `aside`, `nav`

**Table:** `table`, `tr`, `td`, `th`, `thead`, `tbody`, `tfoot`

**Form:** `input`, `button`, `form`, `textarea`, `select`, `option`, `label`

**List:** `ul`, `ol`, `li`, `dl`, `dt`, `dd`

**Text formatting:** `strong`, `em`, `b`, `i`, `u`, `small`, `sub`, `sup`, `pre`, `code`, `blockquote`

**Media:** `img`, `br`, `hr`, `video`, `audio`, `source`, `canvas`, `svg`, `path`

**Meta:** `meta`, `link`, `script`, `style`, `title`

**Interactive:** `details`, `summary`, `dialog`, `figure`, `figcaption`

Any identifier starting with an uppercase letter (e.g., `Button`, `NavBar`, `AppLayout`) is recognized as a **custom component**.

### RSX Example

```volki
pub fn Dashboard(req: &Request) -> Html {
    let user = get_user(req);

    <div class="dashboard" style="padding: 2rem;">
        <nav class="sidebar">
            <a href="/">Home</a>
            <a href="/settings">Settings</a>
        </nav>

        <section class="main-content">
            <h1>{"Welcome, " + user.name}</h1>

            <img src="/avatar.png" alt="Avatar" />
            <input type="text" placeholder="Search..." />

            <UserCard user={user} active={true} />

            <table>
                <thead><tr><th>Name</th><th>Status</th></tr></thead>
                <tbody>
                    {items.iter().map(|item| {
                        <tr>
                            <td>{item.name}</td>
                            <td><span class="badge">{item.status}</span></td>
                        </tr>
                    })}
                </tbody>
            </table>
        </section>

        <footer>
            <small style="color: gray;">{"v" + VERSION}</small>
        </footer>
    </div>
}
```

Inside RSX tags, the plugin highlights:
- **Tag names** (`div`, `span`, `nav`, etc.) in tag color
- **Tag brackets** (`<`, `</`, `>`, `/>`) in bracket color
- **Attributes** (`class`, `style`, `onclick`, etc.) in attribute color
- **String values** (`"dashboard"`, `"Search..."`) in string color
- **Expression values** (`{user.name}`, `{items.iter()...}`) return to Rust mode for full Rust highlighting inside braces

## Compatibility

- IntelliJ Platform 2024.1+
- All JetBrains IDEs based on the IntelliJ Platform
