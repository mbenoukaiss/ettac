use assert_cmd::{Command, cargo_bin};
use predicates::prelude::*;

#[test]
#[cfg_attr(not(feature = "integration"), ignore)]
fn test_symfony_recipe() {
    let expected_output = predicate::str::contains("Script non_existent_recipe.lua does not exist");

    Command::new(cargo_bin!())
        .arg("non_existent_recipe.lua")
        .assert()
        .failure()
        .stderr(expected_output);
}
