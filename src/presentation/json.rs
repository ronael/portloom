use crate::domain::{assessment::assess, model::Snapshot};
use serde_json::{Value, json};
pub fn snapshot(value: &Snapshot) -> Value {
    json!({"capturedAt": value.captured_at, "endpoints": value.endpoints.iter().map(|e| {
        let p = &e.process; let a = assess(p);
        let mut process = json!({"pid":p.pid,"uid":p.uid,"name":p.name,"command":p.command,"startedAt":p.started_at,"cpu":p.cpu,"memoryKb":p.memory_kb});
        if let Some(exe) = &p.executable { process["executable"] = json!(exe); }
        if let Some(cwd) = &p.cwd { process["cwd"] = json!(cwd); }
        if let Some(pr) = &p.project { process["project"] = json!({"name":pr.name,"root":pr.root}); }
        json!({"id":e.id,"port":e.port,"protocol":e.protocol.label(),"addresses":e.addresses,"process":process,
            "assessment":{"kind":a.kind.key(),"label":a.label,"badge":a.badge,"title":a.title,"reason":a.reason,"impact":a.impact,"needsPid":a.needs_pid}})
    }).collect::<Vec<_>>()})
}
