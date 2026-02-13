export type TokenKind =
  | "StringLiteral"
  | "TemplateLiteral"
  | "TemplateHead"
  | "TemplateMiddle"
  | "TemplateTail"
  | "NumericLiteral"
  | "RegexLiteral"
  | "Identifier"
  | "OpenParen"
  | "CloseParen"
  | "OpenBrace"
  | "CloseBrace"
  | "OpenBracket"
  | "CloseBracket"
  | "Semicolon"
  | "Comma"
  | "Dot"
  | "Colon"
  | "QuestionMark"
  | "Arrow"
  | "Spread"
  | "Operator"
  | "Assignment"
  | "LineComment"
  | "BlockComment"
  | "Whitespace"
  | "Newline"
  | "Eof";

export interface Token {
  kind: TokenKind;
  text: string;
  line: number;
  col: number;
}

export interface FormatConfig {
  print_width: number;
  tab_width: number;
  use_tabs: boolean;
  semi: boolean;
  single_quote: boolean;
  bracket_spacing: boolean;
}

export interface Data {
  tokens: Token[];
  config: FormatConfig;
}

export type Options = Record<string, string>;

export interface HandlerResult {
  tokens: Token[];
}

export type HandlerFn = (
  data: Data,
  options: Options
) => HandlerResult | null | Promise<HandlerResult | null>;

export type HookName =
  | "formatter.before_all"
  | "formatter.after_normalize"
  | "formatter.before_whitespace"
  | "formatter.after_all";

export type Handlers = Partial<Record<HookName, HandlerFn>>;

interface PluginRequest {
  version: number;
  hook: string;
  data: Data;
  plugin_options: Options;
}

interface PluginResponse {
  version: 1;
  status: "ok" | "skip" | "error";
  data?: HandlerResult;
  error?: string;
}

export const HOOKS: readonly HookName[] = [
  "formatter.before_all",
  "formatter.after_normalize",
  "formatter.before_whitespace",
  "formatter.after_all",
] as const;

export const TOKEN_KINDS: readonly TokenKind[] = [
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
] as const;

function readStdin(): Promise<string> {
  return new Promise((resolve, reject) => {
    const chunks: string[] = [];
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk: string) => chunks.push(chunk));
    process.stdin.on("end", () => resolve(chunks.join("")));
    process.stdin.on("error", reject);
  });
}

function parseRequest(raw: string): PluginRequest {
  const req = JSON.parse(raw);
  if (typeof req.version !== "number") {
    throw new Error("missing or invalid 'version' field");
  }
  if (typeof req.hook !== "string") {
    throw new Error("missing or invalid 'hook' field");
  }
  if (req.data == null || typeof req.data !== "object") {
    throw new Error("missing or invalid 'data' field");
  }
  return req as PluginRequest;
}

function writeResponse(obj: PluginResponse): void {
  process.stdout.write(JSON.stringify(obj) + "\n");
}

export function createPlugin(handlers: Handlers): void {
  readStdin()
    .then((raw) => {
      const request = parseRequest(raw);
      const handler = handlers[request.hook as HookName];

      if (!handler) {
        writeResponse({ version: 1, status: "skip" });
        return;
      }

      const options = request.plugin_options ?? {};
      return Promise.resolve(handler(request.data, options)).then((result) => {
        if (result == null) {
          writeResponse({ version: 1, status: "skip" });
        } else {
          writeResponse({ version: 1, status: "ok", data: result });
        }
      });
    })
    .catch((err: unknown) => {
      const message = err instanceof Error ? err.message : String(err);
      writeResponse({ version: 1, status: "error", error: message });
      process.exitCode = 1;
    });
}
