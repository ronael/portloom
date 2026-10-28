use crate::application::ports::AppResult;
use std::{
    io::Read,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
const MAX_BYTES: u64 = 8 * 1024 * 1024;
fn read_limited(stream: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    stream
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("System output exceeds the size limit.".into());
    }
    Ok(bytes)
}
pub fn command(program: &str, args: &[&str]) -> AppResult<String> {
    let mut child = Command::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Cannot start {program}: {e}. Check its installation and PATH."))?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let out = thread::spawn(move || read_limited(stdout));
    let err = thread::spawn(move || read_limited(stderr));
    let deadline = Instant::now() + Duration::from_secs(4);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            state => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(match state {
                    Err(e) => format!("{program} : {e}"),
                    _ => format!("{program} : scan exceeded the 4 s deadline."),
                });
            }
        }
    };
    let stdout = out.join().map_err(|_| "stdout reader interrupted.")??;
    let stderr = err.join().map_err(|_| "stderr reader interrupted.")??;
    let status = status?;
    let stdout = String::from_utf8_lossy(&stdout).into_owned();
    let stderr = String::from_utf8_lossy(&stderr);
    if status.success()
        || (status.code() == Some(1) && stdout.trim().is_empty() && stderr.trim().is_empty())
    {
        Ok(stdout)
    } else {
        Err(format!(
            "{program} : {}",
            if stderr.trim().is_empty() {
                format!("failed ({status})")
            } else {
                stderr.trim().to_string()
            }
        )
        .into())
    }
}
