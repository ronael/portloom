use crate::presentation::{
    animation::{AvatarFrame, Mood, Pose},
    design::{ACCENT, CANVAS, DANGER, MUTED, TEXT, WARNING},
};
use ratatui::{Frame, layout::Rect, style::Color};

pub const WIDTH: u16 = 9;
pub const HEIGHT: u16 = 3;
type Pixels = [[u8; WIDTH as usize * 2]; HEIGHT as usize * 2];

// Quarter-cell pixels give the compact silhouette thin eyes, contours and feet.
const BODY: Pixels = [
    *b"........##........",
    *b"....##x####x##....",
    *b"..####x####x####..",
    *b"########xx########",
    *b"....##########....",
    *b".....##....##.....",
];
const QUADRANTS: [&str; 16] = [
    " ", "▘", "▝", "▀", "▖", "▌", "▞", "▛", "▗", "▚", "▐", "▜", "▄", "▙", "▟", "█",
];

fn pixels(frame: AvatarFrame) -> Pixels {
    let mut pixels = BODY;
    if matches!(frame.pose, Pose::Blink | Pose::LookLeft | Pose::LookRight) {
        for row in pixels.iter_mut().take(3).skip(1) {
            row[4..14].fill(b'#');
        }
        if frame.pose == Pose::Blink {
            for x in [5, 6, 11, 12] {
                pixels[2][x] = b'x';
            }
        } else {
            let eyes = if frame.pose == Pose::LookLeft {
                [5, 10]
            } else {
                [7, 12]
            };
            for x in eyes {
                pixels[1][x] = b'x';
                pixels[2][x] = b'x';
            }
        }
    }
    match frame.pose {
        Pose::WaveLow | Pose::WaveHigh => {
            pixels[3][0..4].fill(b'.');
            pixels[2][0..4].fill(b'#');
            pixels[1][0..2].fill(b'#');
            if frame.pose == Pose::WaveHigh {
                pixels[0][0..2].fill(b'#');
            }
        }
        Pose::StepLeft => {
            pixels[5][5..7].fill(b'.');
            pixels[5][4..6].fill(b'#');
        }
        Pose::StepRight => {
            pixels[5][11..13].fill(b'.');
            pixels[5][12..14].fill(b'#');
        }
        _ => {}
    }
    if matches!(frame.mood, Mood::Warning | Mood::Danger) {
        pixels[3][7..11].fill(b'x');
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
            let color = if avatar.pulse && y == 0 && x == 4 {
                TEXT
            } else if y == 0 || x <= 1 {
                mix(body, TEXT)
            } else if y == 2 || x >= 7 {
                mix(body, CANVAS)
            } else {
                body
            };
            frame.buffer_mut()[(area.x + x, area.y + y)]
                .set_symbol(QUADRANTS[mask])
                .set_fg(color)
                .set_bg(CANVAS);
        }
    }
}
