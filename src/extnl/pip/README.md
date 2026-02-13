# volki (Python)

Typed SDK for building [volki](https://github.com/volki/volki) formatter plugins in Python. Zero dependencies, PEP 561 compatible.

## Install

```bash
pip install volki
```

## Quick Start

Create a `volki_plugin.py` in your package:

```python
from volki import create_plugin, Data, Options, HandlerResult

def before_all(data: Data, options: Options) -> HandlerResult | None:
    return {"tokens": data["tokens"]}

create_plugin({
    "formatter.before_all": before_all,
})
```

## API

### `create_plugin(handlers: Handlers) -> None`

Runs the plugin. Reads a JSON request from stdin, dispatches to the matching handler, and writes a JSON response to stdout.

- **`handlers`** — `Dict[HookName, HandlerFn]`: map of hook name to handler function. Unhandled hooks are automatically skipped.

### Types

```python
TokenKind = Literal["StringLiteral", "TemplateLiteral", ..., "Eof"]

class Token(TypedDict):
    kind: TokenKind
    text: str
    line: int
    col: int

class FormatConfig(TypedDict):
    print_width: int
    tab_width: int
    use_tabs: bool
    semi: bool
    single_quote: bool
    bracket_spacing: bool

class Data(TypedDict):
    tokens: List[Token]
    config: FormatConfig

class HandlerResult(TypedDict):
    tokens: List[Token]

Options = Dict[str, str]

HandlerFn = Callable[[Data, Options], Optional[HandlerResult]]

HookName = Literal[
    "formatter.before_all",
    "formatter.after_normalize",
    "formatter.before_whitespace",
    "formatter.after_all",
]
```

PEP 561 `py.typed` marker is included for type checker support.

### Return Values

| Return | Effect |
|--------|--------|
| `{"tokens": [...]}` | Tokens are replaced with the returned list |
| `None` | Hook is skipped, tokens unchanged |
| Raised exception | Reported back to host as an error response |

### Constants

- **`HOOKS`** — `List[HookName]`: list of valid hook names
- **`TOKEN_KINDS`** — `List[TokenKind]`: list of 28 valid token kind strings

## Hook Reference

| Hook | When it runs |
|------|-------------|
| `formatter.before_all` | Before any formatting, right after tokenization |
| `formatter.after_normalize` | After quote/semicolon normalization |
| `formatter.before_whitespace` | After bracket/comma normalization, before whitespace |
| `formatter.after_all` | After all formatting is complete |

## Token Kinds Reference

`StringLiteral`, `TemplateLiteral`, `TemplateHead`, `TemplateMiddle`, `TemplateTail`, `NumericLiteral`, `RegexLiteral`, `Identifier`, `OpenParen`, `CloseParen`, `OpenBrace`, `CloseBrace`, `OpenBracket`, `CloseBracket`, `Semicolon`, `Comma`, `Dot`, `Colon`, `QuestionMark`, `Arrow`, `Spread`, `Operator`, `Assignment`, `LineComment`, `BlockComment`, `Whitespace`, `Newline`, `Eof`

## Configuration

In your project's `volki.toml`:

```toml
[[plugins]]
name = "your-plugin-name"
runtime = "python"

[plugins.options]
some_option = "value"
```

## Error Handling

Any exception raised inside a handler is caught by the SDK and sent back to volki as an error response with `sys.exit(1)`. Use `sys.stderr.write()` for debug logging — stdout is reserved for the protocol.

## Example Plugin

See [`examples/volki_plugin_example/`](./examples/volki_plugin_example/) for a complete working plugin that prepends a banner comment to formatted files.
