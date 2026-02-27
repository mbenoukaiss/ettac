use std::net::TcpStream;
use ssh2::Session;
use crate::context::{AuthMethod, SshCredentials};
use crate::Error;

pub fn login(cred: &SshCredentials) -> Result<Session, Error> {
    let tcp = TcpStream::connect((cred.hostname.as_str(), cred.port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    match &cred.credential {
        AuthMethod::Password(password) => {
            sess.userauth_password(cred.user.as_str(), password)?;
        }
        AuthMethod::PrivateKey(key) => {
            sess.userauth_pubkey_memory(cred.user.as_str(), None, key, None)?;
        }
    };

    if !sess.authenticated() {
        return Err(Error::ssh("Authentication failed"));
    }

    Ok(sess)
}
