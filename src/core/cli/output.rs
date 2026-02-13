use super::style;

pub fn print_banner() {
    eprintln!("{}", style::banner());
}

pub fn print_header(cmd_name: &str) {
    eprintln!("{} volki {}", style::WOLF, style::bold(cmd_name));
}

pub fn print_hint(msg: &str) {
    eprintln!("{}", style::hint(msg));
}

pub fn print_section(title: &str) {
    eprintln!("  {}", style::bold(title));
}

pub fn print_item(symbol: &str, msg: &str) {
    eprintln!("  {} {}", symbol, msg);
}

pub fn print_item_timed(symbol: &str, msg: &str, ms: u128) {
    let dur = style::dim(&style::format_duration(ms));
    eprintln!("  {} {}  {}", symbol, msg, dur);
}

pub fn print_step(i: usize, total: usize, symbol: &str, msg: &str) {
    let counter = style::dim(&format!("[{i}/{total}]"));
    eprintln!("  {} {} {}", counter, symbol, msg);
}

pub fn print_summary_box(lines: &[&str]) {
    if lines.is_empty() {
        return;
    }

    let max_visible_width = lines
        .iter()
        .map(|l| strip_ansi(l).len())
        .max()
        .unwrap_or(0);

    let inner_width = max_visible_width + 2; // 1 space padding on each side

    let top = format!(
        "  \u{250C}{}\u{2510}",
        "\u{2500}".repeat(inner_width)
    );
    let bottom = format!(
        "  \u{2514}{}\u{2518}",
        "\u{2500}".repeat(inner_width)
    );

    eprintln!("{top}");
    for line in lines {
        let visible_len = strip_ansi(line).len();
        let pad = max_visible_width - visible_len;
        eprintln!(
            "  \u{2502} {}{} \u{2502}",
            line,
            " ".repeat(pad),
        );
    }
    eprintln!("{bottom}");
}

/// `aligns`: 'l' (left) or 'r' (right) per column, defaults to 'l'.
pub fn print_table(headers: &[&str], rows: &[Vec<String>], aligns: &[char]) {
    if headers.is_empty() {
        return;
    }

    let cols = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < cols {
                widths[i] = widths[i].max(strip_ansi(cell).len());
            }
        }
    }

    let mut header_parts = Vec::new();
    for (i, h) in headers.iter().enumerate() {
        header_parts.push(format!("{:<width$}", h, width = widths[i]));
    }
    eprintln!("  {}", style::bold(&header_parts.join("  ")));

    let divider_parts: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    eprintln!("  {}", style::dim(&divider_parts.join("  ")));

    for row in rows {
        let mut parts = Vec::new();
        for (i, cell) in row.iter().enumerate() {
            if i < cols {
                let align = aligns.get(i).copied().unwrap_or('l');
                let visible_len = strip_ansi(cell).len();
                let pad = widths[i].saturating_sub(visible_len);
                if align == 'r' {
                    parts.push(format!("{}{}", " ".repeat(pad), cell));
                } else {
                    parts.push(format!("{}{}", cell, " ".repeat(pad)));
                }
            }
        }
        eprintln!("  {}", parts.join("  "));
    }
}

/// Each item is `(depth, text)` where depth 0 = root, 1 = child, etc.
pub fn print_tree(items: &[(usize, &str)]) {
    let total = items.len();
    for (idx, (depth, text)) in items.iter().enumerate() {
        let is_last = idx + 1 >= total
            || items.get(idx + 1).map(|(d, _)| *d <= *depth).unwrap_or(true);

        let indent = if *depth == 0 {
            String::new()
        } else {
            let prefix_spaces = "  ".repeat(depth - 1);
            let connector = if is_last {
                style::TREE_LAST
            } else {
                style::TREE_BRANCH
            };
            format!("{prefix_spaces}{connector} ")
        };

        eprintln!("  {indent}{text}");
    }
}

pub fn print_code_frame(
    lines: &[&str],
    start_line: usize,
    highlight_line: Option<usize>,
    pointer_col: Option<usize>,
    pointer_len: Option<usize>,
) {
    let gutter_width = format!("{}", start_line + lines.len()).len();

    for (i, line) in lines.iter().enumerate() {
        let line_num = start_line + i;
        let is_highlight = highlight_line == Some(line_num);

        let gutter = if is_highlight {
            style::red(&format!("{:>width$}", line_num, width = gutter_width))
        } else {
            style::dim(&format!("{:>width$}", line_num, width = gutter_width))
        };

        let separator = if is_highlight {
            style::red("\u{2502}")
        } else {
            style::dim("\u{2502}")
        };

        eprintln!("    {gutter} {separator} {line}");

        if is_highlight {
            if let (Some(col), Some(len)) = (pointer_col, pointer_len) {
                let spaces = " ".repeat(col);
                let carets = "^".repeat(len);
                let pointer_gutter = " ".repeat(gutter_width);
                eprintln!(
                    "    {pointer_gutter} {} {}",
                    style::red("\u{2502}"),
                    style::red(&format!("{spaces}{carets}"))
                );
            }
        }
    }
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until 'm' (end of SGR sequence)
            for c2 in chars.by_ref() {
                if c2 == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_no_codes() {
        assert_eq!(strip_ansi("hello"), "hello");
    }

    #[test]
    fn strip_ansi_with_codes() {
        assert_eq!(strip_ansi("\x1b[32mhello\x1b[0m"), "hello");
    }

    #[test]
    fn strip_ansi_multiple_codes() {
        assert_eq!(
            strip_ansi("\x1b[1m\x1b[35mtext\x1b[0m"),
            "text"
        );
    }

    #[test]
    fn strip_ansi_empty() {
        assert_eq!(strip_ansi(""), "");
    }
}
