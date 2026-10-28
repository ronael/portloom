use super::ports::{AppResult, Clock, PortScanner, ProcessControl, Signal};
use crate::domain::{
    assessment::{Assessment, assess},
    model::{Endpoint, ProcessInfo, same_process},
    policy::{Actor, blocked},
};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct StopPlan {
    pub target: ProcessInfo,
    pub endpoint_id: String,
    pub endpoints: Vec<Endpoint>,
    pub force: bool,
    pub assessment: Assessment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopResult {
    Stopped,
    Running,
    Gone,
}
pub struct StopProcess {
    scanner: Arc<dyn PortScanner>,
    control: Arc<dyn ProcessControl>,
    actor: Actor,
    clock: Arc<dyn Clock>,
}
impl StopProcess {
    pub fn new(
        scanner: Arc<dyn PortScanner>,
        control: Arc<dyn ProcessControl>,
        actor: Actor,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            scanner,
            control,
            actor,
            clock,
        }
    }
    fn check(&self, target: &ProcessInfo) -> AppResult<()> {
        blocked(target, &self.actor).map_or(Ok(()), |e| Err(e.into()))
    }
    pub fn prepare(&self, endpoint: &Endpoint, force: bool) -> AppResult<StopPlan> {
        self.check(&endpoint.process)?;
        let snapshot = self.scanner.scan()?;
        let current = snapshot
            .endpoints
            .iter()
            .find(|e| e.id == endpoint.id && same_process(&e.process, &endpoint.process))
            .ok_or("Ce port a changé de propriétaire. Actualise la sélection.")?;
        self.check(&current.process)?;
        Ok(StopPlan {
            target: current.process.clone(),
            endpoint_id: current.id.clone(),
            force,
            assessment: assess(&current.process),
            endpoints: snapshot
                .endpoints
                .iter()
                .filter(|e| same_process(&e.process, &current.process))
                .cloned()
                .collect(),
        })
    }
    pub fn execute(&self, plan: &StopPlan, acknowledgement: &str) -> AppResult<StopResult> {
        self.check(&plan.target)?;
        let snapshot = self.scanner.scan()?;
        let Some(current) = snapshot.endpoints.iter().find(|e| e.id == plan.endpoint_id) else {
            return Ok(StopResult::Gone);
        };
        if !same_process(&current.process, &plan.target) {
            return Err("Le processus a changé. Aucun signal envoyé.".into());
        }
        self.check(&current.process)?;
        let assessment = assess(&current.process);
        if assessment.kind != plan.assessment.kind {
            return Err("L’usage du processus a changé. Confirme à nouveau.".into());
        }
        if assessment.needs_pid && acknowledgement != current.process.pid.to_string() {
            return Err("Saisis le PID exact pour confirmer cet arrêt sensible.".into());
        }
        if snapshot
            .endpoints
            .iter()
            .filter(|e| same_process(&e.process, &plan.target))
            .any(|e| !plan.endpoints.iter().any(|old| old.id == e.id))
        {
            return Err(
                "Ce processus a ouvert un autre port. Confirme à nouveau pour voir tous les ports."
                    .into(),
            );
        }
        let Some(live) = self.control.inspect(plan.target.pid)? else {
            return Ok(StopResult::Gone);
        };
        if !same_process(&live, &plan.target) {
            return Err("Le PID a été réutilisé. Aucun signal envoyé.".into());
        }
        self.check(&live)?;
        self.control.signal(
            plan.target.pid,
            if plan.force {
                Signal::Kill
            } else {
                Signal::Terminate
            },
        )?;
        let deadline = self.clock.now().saturating_add(1500);
        loop {
            if !self
                .control
                .inspect(plan.target.pid)?
                .is_some_and(|p| same_process(&p, &plan.target))
            {
                return Ok(StopResult::Stopped);
            }
            if self.clock.now() >= deadline {
                return Ok(StopResult::Running);
            }
            self.clock.sleep(100);
        }
    }
}
