use crate::{
    application::{
        ports::{AppResult, PortScanner},
        preferences::{Language, PreferencesStore},
        stop::{StopPlan, StopProcess, StopResult},
        web::{WebReport, WebService},
    },
    domain::model::{Endpoint, Snapshot},
};
use std::{
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

#[derive(Clone)]
pub struct Services {
    pub scanner: Arc<dyn PortScanner>,
    pub stopper: Arc<StopProcess>,
    pub web: Arc<WebService>,
    pub preferences: Arc<dyn PreferencesStore>,
}
pub(crate) enum Job {
    Scan,
    Prepare(Box<Endpoint>, bool),
    Execute(Box<StopPlan>, String),
    Web(Box<Endpoint>, bool),
    Language(Language),
}
pub(crate) enum Reply {
    Scan(AppResult<Snapshot>),
    Prepare(AppResult<Box<StopPlan>>),
    Execute(AppResult<StopResult>),
    Web(AppResult<Box<WebReport>>),
    Language(Language, AppResult<()>),
}
pub(crate) fn worker(services: Services) -> (Sender<Job>, Receiver<Reply>) {
    let (send, jobs) = mpsc::channel();
    let (replies, recv) = mpsc::channel();
    thread::spawn(move || {
        while let Ok(job) = jobs.recv() {
            let reply = match job {
                Job::Scan => Reply::Scan(services.scanner.scan()),
                Job::Prepare(e, force) => {
                    Reply::Prepare(services.stopper.prepare(&e, force).map(Box::new))
                }
                Job::Execute(plan, ack) => Reply::Execute(services.stopper.execute(&plan, &ack)),
                Job::Web(e, open) => Reply::Web(services.web.check(&e, open).map(Box::new)),
                Job::Language(language) => {
                    Reply::Language(language, services.preferences.save_language(language))
                }
            };
            if replies.send(reply).is_err() {
                break;
            }
        }
    });
    (send, recv)
}
