use assert_cmd::{Command, cargo_bin};
use predicates::prelude::*;

#[test]
#[cfg_attr(not(feature = "integration"), ignore)]
fn test_symfony_recipe() {
    let expected_output = predicate::always()
        .and(predicate::str::starts_with("{"))
        .and(predicate::str::contains("\"prod\": Host {"))
        .and(predicate::str::contains("\"staging\": Host {"))
        .and(predicate::str::ends_with("}\n"));

    Command::new(cargo_bin!())
        .current_dir("tests/symfony")
        .arg("deploy.lua")
        .env("PASSWORD", "y0zHDHwv3X31")
        .env("PRIVATE_KEY", "\
            -----BEGIN OPENSSH PRIVATE KEY-----\n\
            b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW\n\
            QyNTUxOQAAACD0YMsiBzRBn7zzWpcOlK6Xa3Z9QQ0k7nGLtpWDkrxtwwAAAIhMQuxTTELs\n\
            UwAAAAtzc2gtZWQyNTUxOQAAACD0YMsiBzRBn7zzWpcOlK6Xa3Z9QQ0k7nGLtpWDkrxtww\n\
            AAAEA9o3+2y5bV1iyEBsD6clPCJlz66zOEmgOW9KOEqb82fPRgyyIHNEGfvPNalw6Urpdr\n\
            dn1BDSTucYu2lYOSvG3DAAAAA2JvYgEC\n\
            -----END OPENSSH PRIVATE KEY-----\
        ")
        .assert()
        .success()
        .stdout(expected_output);
}
