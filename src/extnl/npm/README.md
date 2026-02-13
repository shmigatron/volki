# volki (Node.js)

TypeScript SDK for building [volki](https://github.com/volki/volki) formatter plugins in Node.js. Zero runtime dependencies.

## Install

```bash
npm install volki
```

## Quick Start

Create a `volki-plugin.ts` in your package root:

```ts
import { createPlugin, Data, Options, HandlerResult } from "volki";

createPlugin({
  "formatter.before_all": (data: Data, options: Options): HandlerResult | null => {
    return { tokens: data.tokens };
  },
});
```

Compile to `volki-plugin.js` (the entry point volki discovers).

## API

### `createPlugin(handlers: Handlers): void`

Runs the plugin. Reads a JSON request from stdin, dispatches to the matching handler, and writes a JSON response to stdout.

- **`handlers`** — `Partial<Record<HookName, HandlerFn>>`: map of hook name to handler function. Unhandled hooks are automatically skipped.

### Types

```ts
type TokenKind = "StringLiteral" | "TemplateLiteral" | ... | "Eof";

interface Token {
  kind: TokenKind;
  text: string;
  line: number;
  col: number;
}

interface FormatConfig {
  print_width: number;
  tab_width: number;
  use_tabs: boolean;
  semi: boolean;
  single_quote: boolean;
  bracket_spacing: boolean;
}

interface Data {
  tokens: Token[];
  config: FormatConfig;
}

type Options = Record<string, string>;

interface HandlerResult {
  tokens: Token[];
}

type HandlerFn = (
  data: Data,
  options: Options
) => HandlerResult | null | Promise<HandlerResult | null>;

type HookName =
  | "formatter.before_all"
  | "formatter.after_normalize"
  | "formatter.before_whitespace"
  | "formatter.after_all";
```

Handlers may be `async` (return a Promise).

### Return Values

| Return | Effect |
|--------|--------|
| `{ tokens: [...] }` | Tokens are replaced with the returned array |
| `null` / `undefined` | Hook is skipped, tokens unchanged |
| Thrown error | Reported back to host as an error response |

### Constants

- **`HOOKS`** — `readonly HookName[]`: list of valid hook names
- **`TOKEN_KINDS`** — `readonly TokenKind[]`: list of 28 valid token kind strings

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
runtime = "node"

[plugins.options]
some_option = "value"
```

## Error Handling

Any exception thrown inside a handler is caught by the SDK and sent back to volki as an error response. Use `process.stderr.write()` for debug logging — stdout is reserved for the protocol.

## Example Plugin

See [`examples/volki-plugin-example/`](./examples/volki-plugin-example/) for a complete working plugin that prepends a banner comment to formatted files.
