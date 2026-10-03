use quanttide_pi::{DOMAIN, VERSION};

#[test]
fn version_matches_manifest() {
    assert_eq!(VERSION, "0.1.0");
}

#[test]
fn domain_name() {
    assert_eq!(DOMAIN, "pi-agent");
}
