const DOMAINS: &[&str] = &["kaspa-ng.org", "kaspa-ng.ip"];

const TLS_DOMAINS: &[&str] = &["kaspa-ng.net"];

pub fn get(tls: bool) -> Vec<String> {
    let fqdns = if tls { TLS_DOMAINS } else { DOMAINS };
    fqdns.iter().map(|fqdn| fqdn.to_string()).collect()
}

pub fn flatten(fqdns: &[String]) -> String {
    fqdns
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}
