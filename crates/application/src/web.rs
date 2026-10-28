use super::ports::{AppResult, PortScanner};
use crate::domain::{
    issue::Issue,
    model::{Endpoint, Snapshot, same_process},
    web::{HttpResponse, HttpTarget},
};
use std::sync::Arc;

pub trait HttpProbe: Send + Sync {
    fn probe(&self, targets: &[HttpTarget]) -> AppResult<HttpResponse>;
}
pub trait BrowserOpener: Send + Sync {
    fn open(&self, target: &HttpTarget) -> AppResult<()>;
}
#[derive(Clone, Debug)]
pub struct WebReport {
    pub endpoint: Endpoint,
    pub response: HttpResponse,
    pub opened: bool,
}
pub struct WebService {
    scanner: Arc<dyn PortScanner>,
    probe: Arc<dyn HttpProbe>,
    browser: Arc<dyn BrowserOpener>,
}
impl WebService {
    pub fn new(
        scanner: Arc<dyn PortScanner>,
        probe: Arc<dyn HttpProbe>,
        browser: Arc<dyn BrowserOpener>,
    ) -> Self {
        Self {
            scanner,
            probe,
            browser,
        }
    }
    pub fn check(&self, selected: &Endpoint, open: bool) -> AppResult<WebReport> {
        let snapshot = self.scanner.scan()?;
        let current = snapshot
            .endpoints
            .iter()
            .find(|e| e.id == selected.id && same_process(&e.process, &selected.process))
            .ok_or(Issue::OwnerChanged)?;
        let response = self
            .probe
            .probe(&unambiguous_targets(&snapshot, current)?)?;
        if open {
            // Recheck identity after the network wait, before opening a browser.
            let fresh = self.scanner.scan()?;
            let live = fresh
                .endpoints
                .iter()
                .find(|e| e.id == current.id && same_process(&e.process, &current.process))
                .ok_or(Issue::OwnerChanged)?;
            if !unambiguous_targets(&fresh, live)?.contains(&response.target) {
                return Err(Issue::OwnerChanged);
            }
            self.browser.open(&response.target)?;
        }
        Ok(WebReport {
            endpoint: current.clone(),
            response,
            opened: open,
        })
    }
}

fn unambiguous_targets(snapshot: &Snapshot, endpoint: &Endpoint) -> AppResult<Vec<HttpTarget>> {
    let mut targets = HttpTarget::for_endpoint(endpoint)?;
    for other in &snapshot.endpoints {
        if other.port == endpoint.port
            && other.process.pid != endpoint.process.pid
            && let Ok(occupied) = HttpTarget::for_endpoint(other)
        {
            targets.retain(|target| !occupied.contains(target));
        }
    }
    if targets.is_empty() {
        return Err(Issue::HttpOwnerAmbiguous);
    }
    Ok(targets)
}
