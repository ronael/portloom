use std::fmt;

/// Stable failure reasons, independent of terminal wording or language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Issue {
    SystemProcess,
    ProtectedProcess,
    OtherUser,
    UnknownIdentity,
    OwnerChanged,
    ProcessChanged,
    UsageChanged,
    ExactPidRequired,
    NewPorts,
    PidReused,
    PortNotFound(u16),
    AmbiguousPort(u16),
    TcpRequired,
    LoopbackRequired,
    HttpUnavailable,
    InvalidHttp,
    HttpHeaderTooLarge,
    HttpOwnerAmbiguous,
    InteractiveRequired,
    Cancelled,
    External(String),
}
impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::SystemProcess => "System process protected: keep this process running.",
            Self::ProtectedProcess => "Portloom and its terminal are protected.",
            Self::OtherUser => "This process belongs to another user.",
            Self::UnknownIdentity => "Process identity unavailable. Refresh the list.",
            Self::OwnerChanged => "This port has changed owner. Refresh the selection.",
            Self::ProcessChanged => "The process changed. No signal was sent.",
            Self::UsageChanged => "The process classification changed. Confirm again.",
            Self::ExactPidRequired => "Enter the exact PID to confirm this sensitive stop.",
            Self::NewPorts => {
                "This process opened another port. Review all ports and confirm again."
            }
            Self::PidReused => "The PID was reused. No signal was sent.",
            Self::PortNotFound(port) => return write!(f, "No matching listener on port {port}."),
            Self::AmbiguousPort(port) => {
                return write!(f, "Port {port} has multiple owners. Choose one with --pid.");
            }
            Self::TcpRequired => "HTTP verification requires a TCP listener.",
            Self::LoopbackRequired => {
                "HTTP verification only connects to local loopback listeners."
            }
            Self::HttpUnavailable => "No HTTP response within 2 seconds. HTTPS is not probed.",
            Self::InvalidHttp => "The listener did not return a valid HTTP/1.x response.",
            Self::HttpHeaderTooLarge => "HTTP headers exceed the 8 KiB limit.",
            Self::HttpOwnerAmbiguous => {
                "This local HTTP address could belong to another process sharing the port. Verification is blocked."
            }
            Self::InteractiveRequired => "An interactive terminal is required for confirmation.",
            Self::Cancelled => "Cancelled. No signal was sent.",
            Self::External(text) => return f.write_str(text),
        };
        f.write_str(text)
    }
}
impl std::error::Error for Issue {}
impl From<String> for Issue {
    fn from(value: String) -> Self {
        Self::External(value)
    }
}
impl From<&str> for Issue {
    fn from(value: &str) -> Self {
        Self::External(value.into())
    }
}
