use assert_cmd::{Command, cargo_bin};

#[test]
#[cfg_attr(not(feature = "integration"), ignore)]
fn test_no_recipe() {
    Command::new(cargo_bin!())
        .arg("--script")
        .arg("non_existent_recipe.lua")
        .assert()
        .failure()
        .stdout("")
        .stderr("no script found at path `\"non_existent_recipe.lua\"`\n");
}
