use crate::{
    application::{
        ports::AppResult,
        web::{BrowserOpener, HttpProbe},
    },
    domain::{
        issue::Issue,
        web::{HttpResponse, HttpTarget},
    },
};
use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub struct LocalHttpProbe {
    budget: Duration,
}
impl Default for LocalHttpProbe {
    fn default() -> Self {
        Self::new(Duration::from_secs(2))
    }
}
impl LocalHttpProbe {
    pub fn new(budget: Duration) -> Self {
        Self { budget }
    }
    fn request(&self, target: &HttpTarget, deadline: Instant) -> AppResult<u16> {
        let remaining = || {
            deadline
                .checked_duration_since(Instant::now())
                .filter(|d| !d.is_zero())
                .ok_or(Issue::HttpUnavailable)
        };
        let ip: IpAddr = target.host().parse().map_err(|_| Issue::LoopbackRequired)?;
        if !ip.is_loopback() {
            return Err(Issue::LoopbackRequired);
        }
        let mut stream =
            TcpStream::connect_timeout(&SocketAddr::new(ip, target.port()), remaining()?)
                .map_err(|_| Issue::HttpUnavailable)?;
        stream
            .set_write_timeout(Some(remaining()?))
            .map_err(|_| Issue::HttpUnavailable)?;
        let request = format!(
            "HEAD / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: Portloom/{}\r\n\r\n",
            target.authority(),
            env!("CARGO_PKG_VERSION")
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|_| Issue::HttpUnavailable)?;
        let mut headers = Vec::new();
        loop {
            stream
                .set_read_timeout(Some(remaining()?))
                .map_err(|_| Issue::HttpUnavailable)?;
            let mut buffer = [0; 1024];
            let count = stream
                .read(&mut buffer)
                .map_err(|_| Issue::HttpUnavailable)?;
            if count == 0 {
                return Err(Issue::InvalidHttp);
            }
            headers.extend_from_slice(&buffer[..count]);
            if let Some(end) = headers.windows(4).position(|w| w == b"\r\n\r\n") {
                if end + 4 > 8192 {
                    return Err(Issue::HttpHeaderTooLarge);
                }
                let line = headers
                    .split(|b| *b == b'\n')
                    .next()
                    .ok_or(Issue::InvalidHttp)?;
                let text = std::str::from_utf8(line).map_err(|_| Issue::InvalidHttp)?;
                let mut fields = text.split_whitespace();
                if !matches!(fields.next(), Some("HTTP/1.0" | "HTTP/1.1")) {
                    return Err(Issue::InvalidHttp);
                }
                let code = fields.next().ok_or(Issue::InvalidHttp)?;
                let status = code.parse::<u16>().map_err(|_| Issue::InvalidHttp)?;
                if code.len() != 3 || !(200..=599).contains(&status) {
                    return Err(Issue::InvalidHttp);
                }
                return Ok(status);
            }
            if headers.len() >= 8192 {
                return Err(Issue::HttpHeaderTooLarge);
            }
        }
    }
}
impl HttpProbe for LocalHttpProbe {
    fn probe(&self, targets: &[HttpTarget]) -> AppResult<HttpResponse> {
        let start = Instant::now();
        let deadline = start + self.budget;
        let mut failure = Issue::HttpUnavailable;
        for target in targets {
            match self.request(target, deadline) {
                Ok(status) => {
                    return Ok(HttpResponse {
                        target: target.clone(),
                        status,
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        simulated: false,
                    });
                }
                Err(error) => failure = error,
            }
        }
        Err(failure)
    }
}
pub struct LocalBrowser;
impl BrowserOpener for LocalBrowser {
    fn open(&self, target: &HttpTarget) -> AppResult<()> {
        let program = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let mut child = Command::new(program)
            .arg(target.url())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                format!(
                    "Cannot start {program}: {e}. Open {} manually.",
                    target.url()
                )
            })?;
        let deadline = Instant::now() + Duration::from_secs(4);
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => return Ok(()),
                Ok(Some(status)) => {
                    return Err(format!(
                        "{program} failed ({status}). Open {} manually.",
                        target.url()
                    )
                    .into());
                }
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "{program} did not finish. Open {} manually.",
                        target.url()
                    )
                    .into());
                }
            }
        }
    }
}
