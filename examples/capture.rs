use portloom::{
    application::{ports::PortScanner, stop::StopProcess},
    domain::policy::Actor,
    infrastructure::{demo::DemoSystem, system::local::clock},
    presentation::{
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
    ] {
        let demo = Arc::new(DemoSystem::default());
        let actor = Actor {
            uid: 501,
            protected_pids: vec![],
        };
        let mut s = State::new(actor.clone(), true);
        s.apply(demo.scan().unwrap());
        s.notice = "Sélectionne un port pour comprendre son usage.".into();
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
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| view::render(f, &mut s)).unwrap();
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
}
