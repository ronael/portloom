use portloom::{
    application::{
        ports::PortScanner,
        stop::{StopProcess, StopResult},
    },
    infrastructure::system::local::{self, LocalControl, LocalScanner},
};
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};
struct Fixture(Child);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
fn finds_tcp_connected_udp_and_stops_only_the_owned_fixture() {
    let script = "import socket,time\nt=socket.socket();t.bind(('127.0.0.1',0));t.listen()\nu=socket.socket(socket.AF_INET,socket.SOCK_DGRAM);u.bind(('127.0.0.1',0));u.connect(('127.0.0.1',9))\nprint(str(t.getsockname()[1])+' '+str(u.getsockname()[1]),flush=True)\ntime.sleep(30)\n";
    let mut fixture = Fixture(
        Command::new("python3")
            .current_dir(std::env::temp_dir())
            .args(["-u", "-c", script])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Python 3 nécessaire pour ce test"),
    );
    let out = fixture.0.stdout.take().unwrap();
    let (send, recv) = mpsc::channel();
    thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(out).read_line(&mut line).map(|_| line);
        let _ = send.send(result);
    });
    let ports: Vec<u16> = recv
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .unwrap()
        .split_whitespace()
        .map(|p| p.parse().unwrap())
        .collect();
    assert_eq!(ports.len(), 2);
    let scanner = Arc::new(LocalScanner::default());
    let snapshot = scanner.scan().unwrap();
    let own: Vec<_> = snapshot
        .endpoints
        .into_iter()
        .filter(|e| e.process.pid == fixture.0.id())
        .collect();
    assert_eq!(own.len(), 2);
    assert!(own.iter().any(|e| e.port == ports[0]));
    assert!(own.iter().any(|e| e.port == ports[1]));
    let stopper = StopProcess::new(
        scanner,
        Arc::new(LocalControl),
        local::actor(),
        local::clock(),
    );
    let plan = stopper.prepare(&own[0], false).unwrap();
    assert_eq!(plan.endpoints.len(), 2);
    assert_eq!(
        stopper.execute(&plan, &fixture.0.id().to_string()).unwrap(),
        StopResult::Stopped
    );
    assert!(fixture.0.wait().is_ok());
}
