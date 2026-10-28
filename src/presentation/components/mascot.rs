use crate::presentation::{
    animation::{AvatarFrame, Mood},
    design::{ACCENT, CANVAS, DANGER, MUTED, TEXT, WARNING, mascot_shadow},
};
use ratatui::{Frame, layout::Rect};

pub const WIDTH: u16 = 16;
pub const HEIGHT: u16 = 8;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

pub fn render(frame: &mut Frame, avatar: AvatarFrame, area: Rect) {
    let body = match avatar.mood {
        Mood::Warning => WARNING,
        Mood::Danger => DANGER,
        Mood::Paused => MUTED,
        _ => ACCENT,
    };
    let shadow = mascot_shadow(body);
    let color = |token| match token {
        b'P' => body,
        b'S' => shadow,
        b'L' => TEXT,
        _ => CANVAS,
    };
    // One matrix column per terminal column, two rows per cell. No resampling.
    let bounds = Rect::new(
        area.x,
        area.y,
        WIDTH.min(area.width),
        HEIGHT.min(area.height),
    )
    .intersection(frame.area());
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let px = usize::from(x - area.x);
            let py = usize::from(y - area.y) * 2;
            frame.buffer_mut()[(x, y)]
                .set_symbol("▀")
                .set_fg(color(avatar.pixels[py][px]))
                .set_bg(color(avatar.pixels[py + 1][px]));
        }
    }
}
