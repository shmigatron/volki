# volki-plugin-example

Example volki formatter plugin that prepends a banner comment to the top of formatted files.

## What it does

- **`formatter.before_all`** — Reads `banner_text` from plugin options and prepends a `// <banner>` line comment followed by a newline. Idempotent: skips if the banner is already present.
- **`formatter.after_all`** — Logs the final token count to stderr (diagnostic only, does not modify tokens).

## Setup

1. Install in your project's virtualenv:

```bash
pip install volki-plugin-example
```

2. Add to `volki.toml`:

```toml
[[plugins]]
name = "volki-plugin-example"
runtime = "python"

[plugins.options]
banner_text = "Auto-generated — do not edit"
```

3. Run the formatter:

```bash
volki format --path src/
```

## Manual testing

```bash
echo '{"version":1,"hook":"formatter.before_all","data":{"tokens":[],"config":{}},"plugin_options":{"banner_text":"Hello"}}' \
  | python3 volki_plugin_example/volki_plugin.py
```

Expected output:

```json
{"version":1,"status":"ok","data":{"tokens":[{"kind":"LineComment","text":"// Hello","line":0,"col":0},{"kind":"Newline","text":"\n","line":0,"col":0}]}}
```
