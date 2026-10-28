use crate::{
    application::{
        ports::{PortScanner, ProcessControl},
        preferences::PreferencesStore,
        stop::StopProcess,
        web::{BrowserOpener, HttpProbe, WebService},
    },
    domain::policy::Actor,
    infrastructure::{
        demo::DemoSystem,
        system::local::{self, LocalControl, LocalScanner},
        web::{LocalBrowser, LocalHttpProbe},
    },
    presentation::tasks::Services,
};
use std::sync::Arc;
struct Adapters {
    scanner: Arc<dyn PortScanner>,
    control: Arc<dyn ProcessControl>,
    probe: Arc<dyn HttpProbe>,
    browser: Arc<dyn BrowserOpener>,
    actor: Actor,
}
pub(super) fn compose(demo: bool, preferences: Arc<dyn PreferencesStore>) -> (Services, Actor) {
    let adapters = if demo {
        let demo = Arc::new(DemoSystem::default());
        Adapters {
            scanner: demo.clone(),
            control: demo.clone(),
            probe: demo.clone(),
            browser: demo,
            actor: Actor {
                uid: 501,
                protected_pids: vec![],
            },
        }
    } else {
        Adapters {
            scanner: Arc::new(LocalScanner::default()),
            control: Arc::new(LocalControl),
            probe: Arc::new(LocalHttpProbe::default()),
            browser: Arc::new(LocalBrowser),
            actor: local::actor(),
        }
    };
    let services = Services {
        stopper: Arc::new(StopProcess::new(
            adapters.scanner.clone(),
            adapters.control,
            adapters.actor.clone(),
            local::clock(),
        )),
        web: Arc::new(WebService::new(
            adapters.scanner.clone(),
            adapters.probe,
            adapters.browser,
        )),
        scanner: adapters.scanner,
        preferences,
    };
    (services, adapters.actor)
}
