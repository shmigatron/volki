use crate::core::volkiwithstds::io::traits::Read;
use crate::vvec;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Space,
    Tab,
    CtrlC,
    Unknown,
}

pub fn read_key() -> Key {
    let stdin = crate::core::volkiwithstds::io::stdin();
    let mut handle = stdin.lock();
    read_key_from(&mut handle)
}

pub fn read_key_from<R: Read>(reader: &mut R) -> Key {
    let mut buf = [0u8; 1];
    if reader.read(&mut buf).unwrap_or(0) == 0 {
        return Key::Unknown;
    }

    match buf[0] {
        0x03 => Key::CtrlC,
        0x09 => Key::Tab,
        0x0A | 0x0D => Key::Enter,
        0x1B => parse_escape(reader),
        0x20 => Key::Space,
        0x7F | 0x08 => Key::Backspace,
        b @ 0x21..=0x7E => Key::Char(b as char),
        b if b >= 0xC0 => parse_utf8(b, reader),
        _ => Key::Unknown,
    }
}

fn parse_escape<R: Read>(reader: &mut R) -> Key {
    let mut buf = [0u8; 1];
    if reader.read(&mut buf).unwrap_or(0) == 0 {
        return Key::Escape;
    }

    if buf[0] != b'[' {
        return Key::Escape;
    }

    if reader.read(&mut buf).unwrap_or(0) == 0 {
        return Key::Escape;
    }

    match buf[0] {
        b'A' => Key::Up,
        b'B' => Key::Down,
        b'C' => Key::Right,
        b'D' => Key::Left,
        _ => Key::Unknown,
    }
}

fn parse_utf8<R: Read>(first: u8, reader: &mut R) -> Key {
    let byte_count = if first & 0xE0 == 0xC0 {
        2
    } else if first & 0xF0 == 0xE0 {
        3
    } else if first & 0xF8 == 0xF0 {
        4
    } else {
        return Key::Unknown;
    };

    let mut bytes = vvec![first];
    for _ in 1..byte_count {
        let mut buf = [0u8; 1];
        if reader.read(&mut buf).unwrap_or(0) == 0 {
            return Key::Unknown;
        }
        bytes.push(buf[0]);
    }

    match core::str::from_utf8(&bytes) {
        Ok(s) => {
            if let Some(c) = s.chars().next() {
                Key::Char(c)
            } else {
                Key::Unknown
            }
        }
        Err(_) => Key::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::io::Cursor;

    #[test]
    fn parse_enter_cr() {
        let mut r = Cursor::new(vvec![0x0D]);
        assert_eq!(read_key_from(&mut r), Key::Enter);
    }

    #[test]
    fn parse_enter_lf() {
        let mut r = Cursor::new(vvec![0x0A]);
        assert_eq!(read_key_from(&mut r), Key::Enter);
    }

    #[test]
    fn parse_space() {
        let mut r = Cursor::new(vvec![0x20]);
        assert_eq!(read_key_from(&mut r), Key::Space);
    }

    #[test]
    fn parse_backspace() {
        let mut r = Cursor::new(vvec![0x7F]);
        assert_eq!(read_key_from(&mut r), Key::Backspace);
    }

    #[test]
    fn parse_ctrl_c() {
        let mut r = Cursor::new(vvec![0x03]);
        assert_eq!(read_key_from(&mut r), Key::CtrlC);
    }

    #[test]
    fn parse_tab() {
        let mut r = Cursor::new(vvec![0x09]);
        assert_eq!(read_key_from(&mut r), Key::Tab);
    }

    #[test]
    fn parse_printable_char() {
        let mut r = Cursor::new(vvec![b'a']);
        assert_eq!(read_key_from(&mut r), Key::Char('a'));
    }

    #[test]
    fn parse_arrow_up() {
        let mut r = Cursor::new(vvec![0x1B, b'[', b'A']);
        assert_eq!(read_key_from(&mut r), Key::Up);
    }

    #[test]
    fn parse_arrow_down() {
        let mut r = Cursor::new(vvec![0x1B, b'[', b'B']);
        assert_eq!(read_key_from(&mut r), Key::Down);
    }

    #[test]
    fn parse_arrow_right() {
        let mut r = Cursor::new(vvec![0x1B, b'[', b'C']);
        assert_eq!(read_key_from(&mut r), Key::Right);
    }

    #[test]
    fn parse_arrow_left() {
        let mut r = Cursor::new(vvec![0x1B, b'[', b'D']);
        assert_eq!(read_key_from(&mut r), Key::Left);
    }

    #[test]
    fn parse_bare_escape() {
        // ESC followed by EOF → bare Escape
        let mut r = Cursor::new(vvec![0x1B]);
        assert_eq!(read_key_from(&mut r), Key::Escape);
    }

    #[test]
    fn parse_escape_non_bracket() {
        let mut r = Cursor::new(vvec![0x1B, b'O']);
        assert_eq!(read_key_from(&mut r), Key::Escape);
    }

    #[test]
    fn parse_utf8_two_byte() {
        // é = 0xC3 0xA9
        let mut r = Cursor::new(vvec![0xC3, 0xA9]);
        assert_eq!(read_key_from(&mut r), Key::Char('é'));
    }

    #[test]
    fn parse_utf8_three_byte() {
        // ✓ = 0xE2 0x9C 0x93
        let mut r = Cursor::new(vvec![0xE2, 0x9C, 0x93]);
        assert_eq!(read_key_from(&mut r), Key::Char('✓'));
    }

    #[test]
    fn parse_utf8_four_byte() {
        // 🐺 = 0xF0 0x9F 0x90 0xBA
        let mut r = Cursor::new(vvec![0xF0, 0x9F, 0x90, 0xBA]);
        assert_eq!(read_key_from(&mut r), Key::Char('🐺'));
    }

    #[test]
    fn parse_empty_input() {
        let mut r = Cursor::new(vvec![]);
        assert_eq!(read_key_from(&mut r), Key::Unknown);
    }

    #[test]
    fn parse_multiple_keys_reads_first() {
        let mut r = Cursor::new(vvec![b'a', b'b']);
        assert_eq!(read_key_from(&mut r), Key::Char('a'));
        assert_eq!(read_key_from(&mut r), Key::Char('b'));
    }
}
