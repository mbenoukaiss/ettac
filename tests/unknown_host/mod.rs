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
        .env("PRIVATE_KEY", "\
            -----BEGIN OPENSSH PRIVATE KEY-----\n\
            b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW\n\
            QyNTUxOQAAACDJme4LaBLGQwPA6qH0G2J13ysV1DSbFrjrpt+cdmK+AgAAAJjlF8fM5RfH\n\
            zAAAAAtzc2gtZWQyNTUxOQAAACDJme4LaBLGQwPA6qH0G2J13ysV1DSbFrjrpt+cdmK+Ag\n\
            AAAEBF07KN5z24CX3MeVVUx7F7nF77CwxV4hKwGqRfRwKRlMmZ7gtoEsZDA8DqofQbYnXf\n\
            KxXUNJsWuOum35x2Yr4CAAAADkNJIGRlcGxveW1lbnRzAQIDBAUGBw==\n\
            -----END OPENSSH PRIVATE KEY-----\
        ")
        .assert()
        .failure()
        .stdout("")
        .stderr("hosts `[\"somethingthadoesntexist\", \"staging\"]` not found\n");
}
