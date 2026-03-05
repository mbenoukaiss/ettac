use crate::BOB_PRIVATE_KEY;
use assert_cmd::{Command, cargo_bin};

#[test]
#[cfg_attr(not(feature = "integration"), ignore)]
fn test_unknown_host() {
    Command::new(cargo_bin!())
        .current_dir("tests/unknown_host")
        .arg("somethingthadoesntexist")
        .arg("prod")
        .arg("staging")
        .env("PASSWORD", "y0zHDHwv3X31")
        .env("PRIVATE_KEY", BOB_PRIVATE_KEY)
        .assert()
        .failure()
        .stdout("")
        .stderr("hosts `[\"somethingthadoesntexist\", \"staging\"]` not found\n");
}
