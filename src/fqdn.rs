const DOMAINS: &[&str] = &[
    "kaspa-ng.net",
    "kaspa.stream",
    "kaspa-ng.org",
    "kaspa-ng.io",
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
