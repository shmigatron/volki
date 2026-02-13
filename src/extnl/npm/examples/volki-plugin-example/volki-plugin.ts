import { createPlugin, Data, Options, HandlerResult, Token } from "volki";

createPlugin({
  "formatter.before_all": (data: Data, options: Options): HandlerResult | null => {
    const banner = options.banner_text ?? "Generated file - do not edit";
    const tokens = data.tokens;

    if (
      tokens.length > 0 &&
      tokens[0].kind === "LineComment" &&
      tokens[0].text === "// " + banner
    ) {
      return null;
    }

    const prefix: Token[] = [
      { kind: "LineComment", text: "// " + banner, line: 0, col: 0 },
      { kind: "Newline", text: "\n", line: 0, col: 0 },
    ];

    return { tokens: [...prefix, ...tokens] };
  },

  "formatter.after_all": (data: Data): null => {
    process.stderr.write(
      `[volki-plugin-example] token count: ${data.tokens.length}\n`
    );
    return null;
  },
});
