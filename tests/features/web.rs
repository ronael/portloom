use portloom::{
    application::{
        ports::{AppResult, PortScanner},
        web::{BrowserOpener, HttpProbe, WebService},
    },
    domain::{
        issue::Issue,
        model::{Endpoint, Protocol, Snapshot},
        web::{HttpResponse, HttpTarget},
    },
    infrastructure::{demo::fixtures, web::LocalHttpProbe},
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

fn endpoint() -> Endpoint {
    fixtures().remove(0)
}
fn serve(response: Vec<u8>, delay: Duration) -> (Endpoint, thread::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut endpoint = endpoint();
    endpoint.port = listener.local_addr().unwrap().port();
    let task = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(2))
                }
                result => panic!("fixture was not contacted: {result:?}"),
            }
        };
        // Darwin can inherit nonblocking mode from the listening socket.
        // The fixture reads a complete request using its bounded read timeout.
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut request = vec![];
        while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let mut bytes = [0; 1024];
            let count = stream.read(&mut bytes).unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&bytes[..count]);
        }
        thread::sleep(delay);
        let _ = stream.write_all(&response);
        request
    });
    (endpoint, task)
}
#[test]
fn http_uses_head_with_numeric_loopback_without_following_redirects() {
    let (endpoint, server) = serve(b"HTTP/1.1 302 Found\r\nLocation: https://example.invalid/private\r\nContent-Length: 0\r\n\r\n".to_vec(), Duration::ZERO);
    let response = LocalHttpProbe::default()
        .probe(&HttpTarget::for_endpoint(&endpoint).unwrap())
        .unwrap();
    assert_eq!(response.status, 302);
    assert_eq!(
        response.target.url(),
        format!("http://127.0.0.1:{}/", endpoint.port)
    );
    let request = String::from_utf8(server.join().unwrap()).unwrap();
    assert!(request.starts_with("HEAD / HTTP/1.1\r\n"));
    assert!(!request.contains("Authorization") && !request.contains("Cookie"));
}
#[test]
fn invalid_large_and_slow_responses_are_bounded() {
    let (endpoint, server) = serve(b"SSH-2.0-test\r\n\r\n".to_vec(), Duration::ZERO);
    assert_eq!(
        LocalHttpProbe::default()
            .probe(&HttpTarget::for_endpoint(&endpoint).unwrap())
            .unwrap_err(),
        Issue::InvalidHttp
    );
    server.join().unwrap();
    let (endpoint, server) = serve(vec![b'a'; 10_000], Duration::ZERO);
    assert_eq!(
        LocalHttpProbe::default()
            .probe(&HttpTarget::for_endpoint(&endpoint).unwrap())
            .unwrap_err(),
        Issue::HttpHeaderTooLarge
    );
    server.join().unwrap();
    let (endpoint, server) = serve(
        b"HTTP/1.1 200 OK\r\n\r\n".to_vec(),
        Duration::from_millis(200),
    );
    let start = Instant::now();
    assert_eq!(
        LocalHttpProbe::new(Duration::from_millis(80))
            .probe(&HttpTarget::for_endpoint(&endpoint).unwrap())
            .unwrap_err(),
        Issue::HttpUnavailable
    );
    assert!(start.elapsed() < Duration::from_secs(1));
    server.join().unwrap();
}
#[test]
fn targets_reject_udp_lan_names_and_url_injection() {
    let mut endpoint = endpoint();
    endpoint.protocol = Protocol::Udp;
    assert_eq!(
        HttpTarget::for_endpoint(&endpoint).unwrap_err(),
        Issue::TcpRequired
    );
    endpoint.protocol = Protocol::Tcp;
    for value in [
        "192.168.1.20",
        "example.com",
        "127.0.0.1@evil.invalid",
        "127.0.0.1/../../",
        "::ffff:192.168.1.20",
    ] {
        endpoint.addresses = vec![value.into()];
        assert_eq!(
            HttpTarget::for_endpoint(&endpoint).unwrap_err(),
            Issue::LoopbackRequired
        );
    }
    endpoint.addresses = vec!["*".into()];
    let targets = HttpTarget::for_endpoint(&endpoint).unwrap();
    assert_eq!(targets.len(), 2);
    assert!(
        targets
            .iter()
            .any(|target| target.url() == "http://[::1]:3000/")
    );
    endpoint.addresses = vec!["127.0.0.2".into()];
    assert_eq!(
        HttpTarget::for_endpoint(&endpoint).unwrap()[0].host(),
        "127.0.0.2"
    );
}
struct Scanner(Mutex<Snapshot>);
impl PortScanner for Scanner {
    fn scan(&self) -> AppResult<Snapshot> {
        Ok(self.0.lock().unwrap().clone())
    }
}
struct Probe {
    scanner: Arc<Scanner>,
    change_owner: bool,
}
impl HttpProbe for Probe {
    fn probe(&self, targets: &[HttpTarget]) -> AppResult<HttpResponse> {
        if self.change_owner {
            self.scanner.0.lock().unwrap().endpoints[0]
                .process
                .started_at = "different process".into();
        }
        Ok(HttpResponse {
            target: targets[0].clone(),
            status: 200,
            elapsed_ms: 1,
            simulated: true,
        })
    }
}
#[derive(Default)]
struct Browser(Mutex<Vec<String>>);
impl BrowserOpener for Browser {
    fn open(&self, target: &HttpTarget) -> AppResult<()> {
        self.0.lock().unwrap().push(target.url());
        Ok(())
    }
}
#[test]
fn ambiguous_socket_ownership_blocks_http_before_network_or_browser_actions() {
    let endpoint = endpoint();
    let mut other = endpoint.clone();
    other.process.pid += 1;
    other.id = "other-owner".into();
    let scanner = Arc::new(Scanner(Mutex::new(Snapshot {
        endpoints: vec![endpoint.clone(), other],
        captured_at: 1,
    })));
    let browser = Arc::new(Browser::default());
    let service = WebService::new(
        scanner.clone(),
        Arc::new(Probe {
            scanner,
            change_owner: false,
        }),
        browser.clone(),
    );
    assert_eq!(
        service.check(&endpoint, true).unwrap_err(),
        Issue::HttpOwnerAmbiguous
    );
    assert!(browser.0.lock().unwrap().is_empty());
}
#[test]
fn opening_rechecks_identity_after_probe_and_verification_alone_never_opens() {
    for change_owner in [false, true] {
        let endpoint = endpoint();
        let scanner = Arc::new(Scanner(Mutex::new(Snapshot {
            endpoints: vec![endpoint.clone()],
            captured_at: 1,
        })));
        let browser = Arc::new(Browser::default());
        let service = WebService::new(
            scanner.clone(),
            Arc::new(Probe {
                scanner,
                change_owner,
            }),
            browser.clone(),
        );
        let result = service.check(&endpoint, true);
        assert_eq!(result.is_err(), change_owner);
        assert_eq!(
            browser.0.lock().unwrap().len(),
            if change_owner { 0 } else { 1 }
        );
    }
    let endpoint = endpoint();
    let scanner = Arc::new(Scanner(Mutex::new(Snapshot {
        endpoints: vec![endpoint.clone()],
        captured_at: 1,
    })));
    let browser = Arc::new(Browser::default());
    let service = WebService::new(
        scanner.clone(),
        Arc::new(Probe {
            scanner,
            change_owner: false,
        }),
        browser.clone(),
    );
    assert!(!service.check(&endpoint, false).unwrap().opened);
    assert!(browser.0.lock().unwrap().is_empty());
}
