"""volki plugin SDK for Python. create_plugin(handlers) handles JSON-over-stdin/stdout IPC."""

from __future__ import annotations

import json
import sys
from typing import (
    Any,
    Callable,
    Dict,
    List,
    Literal,
    Optional,
    TypedDict,
)

TokenKind = Literal[
    "StringLiteral",
    "TemplateLiteral",
    "TemplateHead",
    "TemplateMiddle",
    "TemplateTail",
    "NumericLiteral",
    "RegexLiteral",
    "Identifier",
    "OpenParen",
    "CloseParen",
    "OpenBrace",
    "CloseBrace",
    "OpenBracket",
    "CloseBracket",
    "Semicolon",
    "Comma",
    "Dot",
    "Colon",
    "QuestionMark",
    "Arrow",
    "Spread",
    "Operator",
    "Assignment",
    "LineComment",
    "BlockComment",
    "Whitespace",
    "Newline",
    "Eof",
]


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

Handlers = Dict[HookName, HandlerFn]

HOOKS: List[HookName] = [
    "formatter.before_all",
    "formatter.after_normalize",
    "formatter.before_whitespace",
    "formatter.after_all",
]

TOKEN_KINDS: List[TokenKind] = [
    "StringLiteral",
    "TemplateLiteral",
    "TemplateHead",
    "TemplateMiddle",
    "TemplateTail",
    "NumericLiteral",
    "RegexLiteral",
    "Identifier",
    "OpenParen",
    "CloseParen",
    "OpenBrace",
    "CloseBrace",
    "OpenBracket",
    "CloseBracket",
    "Semicolon",
    "Comma",
    "Dot",
    "Colon",
    "QuestionMark",
    "Arrow",
    "Spread",
    "Operator",
    "Assignment",
    "LineComment",
    "BlockComment",
    "Whitespace",
    "Newline",
    "Eof",
]


class _PluginRequest(TypedDict):
    version: int
    hook: str
    data: Data
    plugin_options: Options


def _read_stdin() -> str:
    return sys.stdin.read()


def _parse_request(raw: str) -> _PluginRequest:
    req: Dict[str, Any] = json.loads(raw)
    if not isinstance(req.get("version"), (int, float)):
        raise ValueError("missing or invalid 'version' field")
    if not isinstance(req.get("hook"), str):
        raise ValueError("missing or invalid 'hook' field")
    if not isinstance(req.get("data"), dict):
        raise ValueError("missing or invalid 'data' field")
    return req  # type: ignore[return-value]


def _write_response(obj: Dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(obj) + "\n")
    sys.stdout.flush()


def create_plugin(handlers: Handlers) -> None:
    """Run a volki plugin with the given hook handlers.

    Args:
        handlers: Map of hook name to handler function.

    Example::

        from volki import create_plugin, Data, Options, HandlerResult

        def before_all(data: Data, options: Options) -> HandlerResult | None:
            return {"tokens": data["tokens"]}

        create_plugin({
            "formatter.before_all": before_all,
        })
    """
    try:
        raw = _read_stdin()
        request = _parse_request(raw)
        handler = handlers.get(request["hook"])  # type: ignore[arg-type]

        if handler is None:
            _write_response({"version": 1, "status": "skip"})
            return

        options: Options = request.get("plugin_options") or {}
        result = handler(request["data"], options)

        if result is None:
            _write_response({"version": 1, "status": "skip"})
        else:
            _write_response({"version": 1, "status": "ok", "data": result})

    except Exception as exc:
        _write_response({
            "version": 1,
            "status": "error",
            "error": str(exc),
        })
        sys.exit(1)
