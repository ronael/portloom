use crate::presentation::{
    animation::{AvatarFrame, Mood, Pose},
    design::{ACCENT, CANVAS, DANGER, MUTED, TEXT, WARNING},
};
use ratatui::{Frame, layout::Rect, style::Color};

pub const WIDTH: u16 = 9;
pub const HEIGHT: u16 = 3;
type Pixels = [[u8; WIDTH as usize * 2]; HEIGHT as usize * 2];

// A broad shell leaves room for the face; hands move outside its contour.
// Quarter-cell pixels keep the rounded corners and contact-pin feet compact.
const BODY: Pixels = [
    *b"...############...",
    *b"..###xx####xx###..",
    *b"..###xx####xx###..",
    *b"##################",
    *b"...############...",
    *b"....##......##....",
];
const QUADRANTS: [&str; 16] = [
    " ", "▘", "▝", "▀", "▖", "▌", "▞", "▛", "▗", "▚", "▐", "▜", "▄", "▙", "▟", "█",
];

fn pixels(frame: AvatarFrame) -> Pixels {
    let mut pixels = BODY;
    if matches!(
        frame.pose,
        Pose::Blink | Pose::Wink | Pose::LookLeft | Pose::LookRight
    ) {
        for row in pixels.iter_mut().take(3).skip(1) {
            row[4..14].fill(b'#');
        }
        let eyes = match frame.pose {
            Pose::LookLeft => [4, 10],
            Pose::LookRight => [6, 12],
            _ => [5, 11],
        };
        for (index, x) in eyes.into_iter().enumerate() {
            pixels[2][x..x + 2].fill(b'x');
            if frame.pose != Pose::Blink && !(frame.pose == Pose::Wink && index == 0) {
                pixels[1][x..x + 2].fill(b'x');
            }
        }
    }
    match frame.pose {
        Pose::WaveLow | Pose::WaveHigh => {
            pixels[3][0..2].fill(b'.');
            pixels[2][0..2].fill(b'#');
            pixels[1][0] = b'#';
            if frame.pose == Pose::WaveHigh {
                pixels[0][0] = b'#';
            }
        }
        Pose::StepLeft => {
            pixels[5][4..6].fill(b'.');
            pixels[5][3..5].fill(b'#');
        }
        Pose::StepRight => {
            pixels[5][12..14].fill(b'.');
            pixels[5][13..15].fill(b'#');
        }
        _ => {}
    }
    if matches!(frame.mood, Mood::Warning | Mood::Danger) {
        pixels[3][8..10].fill(b'x');
    }
    pixels
}
fn mix(base: Color, toward: Color) -> Color {
    if let (Color::Rgb(r, g, b), Color::Rgb(tr, tg, tb)) = (base, toward) {
        let blend = |a, b| ((u16::from(a) * 2 + u16::from(b)) / 3) as u8;
        Color::Rgb(blend(r, tr), blend(g, tg), blend(b, tb))
    } else {
        base
    }
}
pub fn render(frame: &mut Frame, avatar: AvatarFrame, area: Rect) {
    let body = match avatar.mood {
        Mood::Warning => WARNING,
        Mood::Danger => DANGER,
        Mood::Paused => MUTED,
        _ => ACCENT,
    };
    let pixels = pixels(avatar);
    let area = area.intersection(frame.area());
    for y in 0..HEIGHT.min(area.height) {
        for x in 0..WIDTH.min(area.width) {
            let px = usize::from(x) * 2;
            let py = usize::from(y) * 2;
            let mut mask = 0;
            for (bit, dx, dy) in [(1, 0, 0), (2, 1, 0), (4, 0, 1), (8, 1, 1)] {
                if pixels[py + dy][px + dx] == b'#' {
                    mask |= bit;
                }
            }
            let light = mix(body, TEXT);
            let shade = mix(body, CANVAS);
            let (glyph, foreground, background) = match (y, mask) {
                // Two ink colors within a full cell give the brow a thin highlight.
                (0, 15) => ("▀", if avatar.pulse && x == 4 { TEXT } else { light }, body),
                (2, 15) => ("▀", body, shade),
                (2, _) => (QUADRANTS[mask], shade, CANVAS),
                _ => (QUADRANTS[mask], body, CANVAS),
            };
            frame.buffer_mut()[(area.x + x, area.y + y)]
                .set_symbol(glyph)
                .set_fg(foreground)
                .set_bg(background);
        }
    }
}
