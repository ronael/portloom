use ratatui::style::{Color, Style};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
pub const CANVAS: Color = Color::Rgb(16, 18, 22);
pub const SURFACE: Color = Color::Rgb(23, 26, 32);
pub const RAISED: Color = Color::Rgb(32, 36, 44);
pub const SELECTED: Color = Color::Rgb(36, 51, 56);
pub const TEXT: Color = Color::Rgb(231, 233, 238);
pub const MUTED: Color = Color::Rgb(155, 166, 183);
pub const SUBTLE: Color = Color::Rgb(121, 133, 152);
pub const BORDER: Color = Color::Rgb(50, 58, 71);
pub const ACCENT: Color = Color::Rgb(121, 219, 200);
pub const SUCCESS: Color = Color::Rgb(163, 217, 165);
pub const WARNING: Color = Color::Rgb(242, 196, 123);
pub const DANGER: Color = Color::Rgb(242, 143, 149);
pub fn mascot_shadow(body: Color) -> Color {
    if let (Color::Rgb(r, g, b), Color::Rgb(cr, cg, cb)) = (body, CANVAS) {
        let channel = |base, canvas| ((u16::from(base) * 2 + u16::from(canvas)) / 3) as u8;
        Color::Rgb(channel(r, cr), channel(g, cg), channel(b, cb))
    } else {
        body
    }
}
pub fn style(color: Color) -> Style {
    Style::default().fg(color)
}
pub fn safe(value: &str) -> String {
    // Remove complete escape sequences before filtering control / bidi characters.
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            match chars.next() {
                Some('[') => {
                    for x in chars.by_ref() {
                        if ('@'..='~').contains(&x) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    while let Some(x) = chars.next() {
                        if x == '\u{7}' || (x == '\u{1b}' && chars.peek() == Some(&'\\')) {
                            if x == '\u{1b}' {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        if c.is_control()
            || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{200e}' | '\u{200f}' | '\u{061c}')
        {
            if c == '\n' || c == '\t' {
                out.push(' ');
            }
            continue;
        }
        out.push(c);
    }
    out
}
pub fn fit(value: &str, width: usize) -> String {
    let clean = safe(value);
    if clean.width() <= width {
        return clean;
    }
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for g in clean.graphemes(true) {
        let w = g.width();
        if used + w > width - 1 {
            break;
        }
        used += w;
        out.push_str(g);
    }
    out.push('…');
    out
}

pub fn wrap(value: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let text = safe(value);
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.width() + 1 + word.width() <= width {
            line.push(' ');
            line.push_str(word);
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        for g in word.graphemes(true) {
            if !line.is_empty() && line.width() + g.width() > width {
                lines.push(std::mem::take(&mut line));
            }
            line.push_str(g);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
