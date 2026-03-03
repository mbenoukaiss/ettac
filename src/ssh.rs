use libssh_rs::{Session, SshKey, SshOption};
use crate::context::{AuthMethod, SshCredentials};
use crate::Error;

pub fn login(cred: &SshCredentials) -> Result<Session, Error> {
    let sess = Session::new()?;
    sess.set_option(SshOption::Hostname(cred.hostname.clone()))?;
    sess.set_option(SshOption::Port(cred.port))?;
    sess.set_option(SshOption::User(Some(cred.user.clone())))?;
    sess.options_parse_config(None)?;
    sess.connect()?;

    match &cred.credential {
        AuthMethod::Password(password) => {
            sess.userauth_password(None, Some(password))?;
        }
        AuthMethod::Key(key, passphrase) => {
            let key = SshKey::from_privkey_base64(key, passphrase.as_ref().map(String::as_str))?;

            sess.userauth_publickey(
                None,
                &key,
            )?;
        }
    };

    Ok(sess)
}
