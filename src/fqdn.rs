const DOMAINS: &[&str] = &[
    "kaspa.stream",
    "kaspa.red",
    "kaspa.green",
    "kaspa.blue",
    "kaspa-ng.net",
    "kaspa-ng.org",
    "kaspa-ng.io",
    "kaspa.org",
    "kaspacalc.net",
];

pub fn get() -> Vec<String> {
    DOMAINS.iter().map(|fqdn| format!("*.{fqdn}")).collect()
}

pub fn flatten(fqdns: &[String]) -> String {
    fqdns
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}
