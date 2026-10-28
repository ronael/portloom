use super::{
    issue::Issue,
    model::{Endpoint, Protocol},
};

/// Constructed only from a listener's numeric loopback/wildcard addresses.
/// No arbitrary URL, DNS name, credentials, path, or redirect can enter this type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HttpTarget {
    host: String,
    port: u16,
}
impl HttpTarget {
    pub fn for_endpoint(endpoint: &Endpoint) -> Result<Vec<Self>, Issue> {
        if endpoint.protocol != Protocol::Tcp {
            return Err(Issue::TcpRequired);
        }
        let mut hosts = Vec::new();
        for address in &endpoint.addresses {
            let address = address.trim_matches(['[', ']']);
            match address {
                "*" => {
                    hosts.push("127.0.0.1".to_owned());
                    hosts.push("::1".to_owned());
                }
                "0.0.0.0" => hosts.push("127.0.0.1".into()),
                "::" | "::1" => hosts.push("::1".into()),
                _ => {
                    let octets = address
                        .split('.')
                        .map(str::parse::<u8>)
                        .collect::<Result<Vec<_>, _>>();
                    if let Ok(octets) = octets
                        && octets.len() == 4
                        && octets[0] == 127
                    {
                        hosts.push(
                            octets
                                .iter()
                                .map(u8::to_string)
                                .collect::<Vec<_>>()
                                .join("."),
                        );
                    }
                }
            }
        }
        hosts.sort();
        hosts.dedup();
        if hosts.is_empty() || endpoint.port == 0 {
            return Err(Issue::LoopbackRequired);
        }
        Ok(hosts
            .into_iter()
            .map(|host| Self {
                host,
                port: endpoint.port,
            })
            .collect())
    }
    pub fn host(&self) -> &str {
        &self.host
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn authority(&self) -> String {
        if self.host.contains(':') {
            format!("[{}]:{}", self.host, self.port)
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }
    pub fn url(&self) -> String {
        format!("http://{}/", self.authority())
    }
}
#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub target: HttpTarget,
    pub status: u16,
    pub elapsed_ms: u64,
    pub simulated: bool,
}
