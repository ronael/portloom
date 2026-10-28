use super::i18n::Locale;
use crate::domain::{assessment::assess, model::Snapshot};
use serde_json::{Value, json};
pub fn snapshot(value: &Snapshot) -> Value {
    localized_snapshot(value, Locale::EN)
}
pub fn localized_snapshot(value: &Snapshot, locale: Locale) -> Value {
    json!({"capturedAt": value.captured_at, "endpoints": value.endpoints.iter().map(|e| {
        let p = &e.process; let a = assess(p); let text = locale.assessment(&a);
        let mut process = json!({"pid":p.pid,"uid":p.uid,"name":p.name,"command":p.command,"startedAt":p.started_at,"cpu":p.cpu,"memoryKb":p.memory_kb});
        if let Some(exe) = &p.executable { process["executable"] = json!(exe); }
        if let Some(cwd) = &p.cwd { process["cwd"] = json!(cwd); }
        if let Some(pr) = &p.project { process["project"] = json!({"name":pr.name,"root":pr.root,"branch":pr.branch}); }
        json!({"id":e.id,"port":e.port,"protocol":e.protocol.label(),"addresses":e.addresses,"process":process,
            "assessment":{"kind":a.kind.key(),"label":text.label,"badge":text.badge,"title":a.title,"reason":text.reason,"impact":text.impact,"needsPid":a.needs_pid}})
    }).collect::<Vec<_>>()})
}
