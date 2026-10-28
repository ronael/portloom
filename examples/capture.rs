use portloom::{
    application::{ports::PortScanner, stop::StopProcess},
    domain::policy::Actor,
    infrastructure::{demo::DemoSystem, system::local::clock},
    presentation::{
        animation::{AvatarFrame, Mood, Sequence},
        i18n::{Locale, T},
        state::{Overlay, State},
        view,
    },
};
use ratatui::{Terminal, backend::TestBackend, style::Color};
use std::{fs, sync::Arc};
fn color(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        _ => "#101216".into(),
    }
}
fn main() {
    let root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".tooling/previews".into());
    fs::create_dir_all(&root).unwrap();
    for (name, w, h, view_index, dialog) in [
        ("wide", 140, 40, 0, false),
        ("compact", 80, 24, 0, false),
        ("minimum", 60, 20, 0, false),
        ("system", 140, 40, 4, false),
        ("confirmation", 80, 24, 2, true),
        ("groups", 140, 40, 1, false),
        ("french", 140, 40, 1, false),
        ("language", 80, 24, 0, false),
        ("paused", 80, 24, 0, false),
        ("busy", 80, 24, 0, false),
        ("error", 80, 24, 0, false),
    ] {
        let demo = Arc::new(DemoSystem::default());
        let actor = Actor {
            uid: 501,
            protected_pids: vec![],
        };
        let mut s = State::new(actor.clone(), true);
        s.apply(demo.scan().unwrap());
        if name == "french" {
            s.locale = Locale::FR;
        }
        s.notice = s.locale.text(T::SelectPort).into();
        match name {
            "paused" => {
                s.demo = false;
                s.paused = true;
            }
            "busy" => {
                s.demo = false;
                s.busy = true;
            }
            "error" => {
                s.demo = false;
                s.stale = true;
                s.error = true;
            }
            _ => {}
        }
        if view_index == 1 {
            s.grouped = true;
        }
        s.view = view_index;
        s.rebuild();
        if dialog {
            let stop = StopProcess::new(demo.clone(), demo, actor, clock());
            s.overlay = Overlay::Confirm {
                plan: Box::new(stop.prepare(s.selected().unwrap(), false).unwrap()),
                input: String::new(),
                scroll: 0,
            };
        }
        if name == "language" {
            s.overlay = Overlay::Language {
                selected: portloom::application::preferences::Language::French,
            };
        }
        s.animate(1500);
        capture(&root, name, w, h, &mut s);
    }
    let demo = DemoSystem::default();
    let mut s = State::new(
        Actor {
            uid: 501,
            protected_pids: vec![],
        },
        true,
    );
    s.apply(demo.scan().unwrap());
    let mut index = 0;
    for (sequence, mood) in [
        (Sequence::Arrival, Mood::Idle),
        (Sequence::Rest, Mood::Idle),
        (Sequence::Listening, Mood::Working),
        (Sequence::Alert, Mood::Warning),
        (Sequence::Departure, Mood::Idle),
    ] {
        for sprite in sequence.frames() {
            s.avatar_frame = AvatarFrame {
                mood,
                pixels: &sprite.pixels,
            };
            let name = format!("motion-{index:03}");
            capture(&root, &name, 80, 24, &mut s);
            fs::write(
                format!("{root}/{name}/duration.json"),
                sprite.duration.to_string(),
            )
            .unwrap();
            index += 1;
        }
    }
}
fn capture(root: &str, name: &str, w: u16, h: u16, s: &mut State) {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal.draw(|f| view::render(f, s)).unwrap();
    let b = terminal.backend().buffer();
    let rows = (0..h)
        .map(|y| {
            (0..w)
                .map(|x| {
                    let c = &b[(x, y)];
                    serde_json::json!({"text":c.symbol(),"fg":color(c.fg),"bg":color(c.bg)})
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let dir = format!("{root}/{name}");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        format!("{dir}/frame.json"),
        serde_json::to_vec(&rows).unwrap(),
    )
    .unwrap();
    fs::write(
        format!("{dir}/frame.txt"),
        (0..h)
            .map(|y| (0..w).map(|x| b[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
}
