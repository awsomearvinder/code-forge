pub struct SshServer;
impl SshServer {
    pub fn new() -> Self {
        Self
    }
}

impl russh::server::Server for SshServer {
    type Handler = SshHandler;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        Self::Handler::new()
    }
}

pub struct SshHandler;
impl SshHandler {
    pub fn new() -> Self {
        Self
    }
}

pub enum SshHandlerErr {
    Disconnect,
}
impl From<russh::Error> for SshHandlerErr {
    fn from(_value: russh::Error) -> Self {
        todo!()
    }
}

impl russh::server::Handler for SshHandler {
    type Error = SshHandlerErr;
    async fn auth_none(&mut self, _user: &str) -> Result<russh::server::Auth, Self::Error> {
        Ok(russh::server::Auth::Accept)
    }
    async fn auth_password(
        &mut self,
        _user: &str,
        _password: &str,
    ) -> Result<russh::server::Auth, Self::Error> {
        Ok(russh::server::Auth::Accept)
    }
    async fn auth_publickey_offered(
        &mut self,
        _user: &str,
        _public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<russh::server::Auth, Self::Error> {
        Ok(russh::server::Auth::Accept)
    }
    async fn data(
        &mut self,
        channel: russh::ChannelId,
        data: &[u8],
        session: &mut russh::server::Session,
    ) -> Result<(), Self::Error> {
        if data == [3] {
            return Err(SshHandlerErr::Disconnect);
        }
        println!("{}", String::from_utf8_lossy(data));
        Ok(())
    }
    async fn channel_open_session(
        &mut self,
        channel: russh::Channel<russh::server::Msg>,
        session: &mut russh::server::Session,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
    async fn channel_close(
        &mut self,
        channel: russh::ChannelId,
        session: &mut russh::server::Session,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
