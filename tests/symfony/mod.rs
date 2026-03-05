use crate::BOB_PRIVATE_KEY;
use assert_cmd::{Command, cargo_bin};
use predicates::prelude::*;

#[test]
#[cfg_attr(not(feature = "integration"), ignore)]
fn test_symfony_recipe() {
    let expected_output = predicate::always()
        .and(predicate::str::contains("Deploying host prod"))
        .and(predicate::str::contains("Deploying host staging"));

    Command::new(cargo_bin!())
        .current_dir("tests/symfony")
        .arg("prod")
        .arg("staging")
        .env("PASSWORD", "y0zHDHwv3X31")
        .env("PRIVATE_KEY", BOB_PRIVATE_KEY)
        .assert()
        .success()
        .stdout(expected_output);
}
