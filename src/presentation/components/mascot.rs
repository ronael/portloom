use crate::presentation::{
    animation::{AvatarFrame, Mood, Pose},
    design::{ACCENT, CANVAS, DANGER, MUTED, TEXT, WARNING},
};
use ratatui::{Frame, layout::Rect, style::Color};

pub const WIDTH: u16 = 13;
pub const HEIGHT: u16 = 4;
type Pixels = [[u8; WIDTH as usize]; HEIGHT as usize * 2];

// Two square pixels per terminal cell: a solid shell, antenna, eyes, arms and pins.
const BODY: Pixels = [
    *b".....+#+.....",
    *b"...+#####+...",
    *b"..+#######=..",
    *b".+##xx#xx##=.",
    *b"==##xx#xx##==",
    *b"..=###x###=..",
    *b"...=#...#=...",
    *b".............",
];

fn pixels(frame: AvatarFrame) -> Pixels {
    let mut pixels = BODY;
    if matches!(frame.pose, Pose::Blink | Pose::LookLeft | Pose::LookRight) {
        for row in pixels.iter_mut().take(5).skip(3) {
            row[3..10].fill(b'#');
        }
        let eyes = match frame.pose {
            Pose::LookLeft => [3, 4, 6, 7],
            Pose::LookRight => [5, 6, 8, 9],
            _ => [4, 5, 7, 8],
        };
        for x in eyes {
            pixels[4][x] = b'x';
            if frame.pose != Pose::Blink {
                pixels[3][x] = b'x';
            }
        }
    }
    match frame.pose {
        Pose::WaveLow | Pose::WaveHigh => {
            pixels[4][0..2].fill(b'.');
            pixels[3][0..2].fill(b'#');
            pixels[2][0] = b'+';
            pixels[2][1] = b'#';
            if frame.pose == Pose::WaveHigh {
                pixels[1][0] = b'+';
                pixels[1][1] = b'#';
            }
        }
        Pose::StepLeft => {
            pixels[6][3..5].fill(b'.');
            pixels[6][2..4].fill(b'#');
        }
        Pose::StepRight => {
            pixels[6][8..10].fill(b'.');
            pixels[6][9..11].fill(b'#');
        }
        _ => {}
    }
    if matches!(frame.mood, Mood::Warning | Mood::Danger) {
        for row in pixels.iter_mut().take(5).skip(3) {
            row[5] = b'#';
            row[7] = b'#';
        }
        pixels[5][5..8].fill(b'x');
    }
    if frame.pulse {
        pixels[0][6] = b'*';
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
    let color = |pixel| match pixel {
        b'#' => body,
        b'+' => mix(body, TEXT),
        b'=' => mix(body, CANVAS),
        b'*' => TEXT,
        _ => CANVAS,
    };
    let pixels = pixels(avatar);
    let area = area.intersection(frame.area());
    for y in 0..HEIGHT.min(area.height) {
        for x in 0..WIDTH.min(area.width) {
            let top = color(pixels[usize::from(y) * 2][usize::from(x)]);
            let bottom = color(pixels[usize::from(y) * 2 + 1][usize::from(x)]);
            let cell = &mut frame.buffer_mut()[(area.x + x, area.y + y)];
            if top == bottom {
                cell.set_symbol(" ").set_bg(top);
            } else {
                cell.set_symbol("▀").set_fg(top).set_bg(bottom);
            }
        }
    }
}
