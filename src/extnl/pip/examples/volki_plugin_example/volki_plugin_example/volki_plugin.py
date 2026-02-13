"""Example volki plugin that prepends a banner comment."""

from __future__ import annotations

import sys

from volki import create_plugin, Data, Options, HandlerResult, Token


def before_all(data: Data, options: Options) -> HandlerResult | None:
    banner = options.get("banner_text", "Generated file - do not edit")
    tokens = data["tokens"]

    if (
        len(tokens) > 0
        and tokens[0]["kind"] == "LineComment"
        and tokens[0]["text"] == "// " + banner
    ):
        return None

    prefix: list[Token] = [
        {"kind": "LineComment", "text": "// " + banner, "line": 0, "col": 0},
        {"kind": "Newline", "text": "\n", "line": 0, "col": 0},
    ]

    return {"tokens": prefix + tokens}


def after_all(data: Data, options: Options) -> None:
    sys.stderr.write(
        f"[volki-plugin-example] token count: {len(data['tokens'])}\n"
    )
    return None


create_plugin({
    "formatter.before_all": before_all,
    "formatter.after_all": after_all,
})
