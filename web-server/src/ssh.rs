use std::{collections::HashMap, io::Write, path::PathBuf};

use russh::{Channel, ChannelId};

pub struct SshServer {
    data_dir: PathBuf,
}
impl SshServer {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir: data_dir }
    }
}

impl russh::server::Server for SshServer {
    type Handler = SshHandler;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        Self::Handler::new(self.data_dir.clone())
    }
}

enum PackWorkerMsg {
    Data(russh::CryptoVec),
    Eof,
}
pub struct SshHandler {
    channels: std::sync::Arc<tokio::sync::Mutex<HashMap<ChannelId, Channel<russh::server::Msg>>>>,
    data_dir: PathBuf,
}

impl SshHandler {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            channels: Default::default(),
            data_dir,
        }
    }
}

pub enum SshHandlerErr {
    Disconnect,
    UnknownCommand,
    ChannelNotFound,
    FailedToOpenRepo,
}
impl From<russh::Error> for SshHandlerErr {
    fn from(_value: russh::Error) -> Self {
        match _value {
            russh::Error::CouldNotReadKey => todo!(),
            russh::Error::KexInit => todo!(),
            russh::Error::UnknownAlgo => todo!(),
            russh::Error::NoCommonAlgo { .. } => todo!(),
            russh::Error::Version => todo!(),
            russh::Error::Kex => todo!(),
            russh::Error::PacketAuth => todo!(),
            russh::Error::Inconsistent => todo!(),
            russh::Error::NotAuthenticated => todo!(),
            russh::Error::IndexOutOfBounds => todo!(),
            russh::Error::UnknownKey => todo!(),
            russh::Error::WrongServerSig => todo!(),
            russh::Error::PacketSize(_) => todo!(),
            russh::Error::WrongChannel => todo!(),
            russh::Error::ChannelOpenFailure(..) => todo!(),
            russh::Error::Disconnect => todo!(),
            russh::Error::NoHomeDir => todo!(),
            russh::Error::KeyChanged { .. } => todo!(),
            russh::Error::HUP => todo!(),
            russh::Error::ConnectionTimeout => todo!(),
            russh::Error::KeepaliveTimeout => todo!(),
            russh::Error::InactivityTimeout => todo!(),
            russh::Error::NoAuthMethod => todo!(),
            russh::Error::SendError => todo!(),
            russh::Error::Pending => todo!(),
            russh::Error::DecryptionError => todo!(),
            russh::Error::RequestDenied => todo!(),
            russh::Error::Keys(_error) => todo!(),
            russh::Error::IO(_error) => todo!(),
            russh::Error::Utf8(_utf8_error) => todo!(),
            russh::Error::Compress(_compress_error) => todo!(),
            russh::Error::Decompress(_decompress_error) => todo!(),
            russh::Error::Join(_join_error) => todo!(),
            russh::Error::Elapsed(_elapsed) => todo!(),
            russh::Error::StrictKeyExchangeViolation { .. } => todo!(),
            russh::Error::Signature(_error) => todo!(),
            russh::Error::SshKey(_error) => todo!(),
            russh::Error::SshEncoding(_error) => todo!(),
            russh::Error::InvalidConfig(_) => todo!(),
        }
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
        _channel: russh::ChannelId,
        data: &[u8],
        _session: &mut russh::server::Session,
    ) -> Result<(), Self::Error> {
        if data == [3] {
            return Err(SshHandlerErr::Disconnect);
        }
        println!("data {}", String::from_utf8_lossy(data));
        Ok(())
    }
    async fn extended_data(
        &mut self,
        _channel: russh::ChannelId,
        _code: u32,
        data: &[u8],
        _session: &mut russh::server::Session,
    ) -> Result<(), Self::Error> {
        if data == [3] {
            return Err(SshHandlerErr::Disconnect);
        }
        println!("extended data: {}", String::from_utf8_lossy(data));
        Ok(())
    }
    async fn channel_open_session(
        &mut self,
        channel: russh::Channel<russh::server::Msg>,
        _session: &mut russh::server::Session,
    ) -> Result<bool, Self::Error> {
        self.channels.lock().await.insert(channel.id(), channel);
        println!("channel_open_session: ");
        Ok(true)
    }
    async fn channel_close(
        &mut self,
        _channel: russh::ChannelId,
        _session: &mut russh::server::Session,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel_id: russh::ChannelId,
        data: &[u8],
        session: &mut russh::server::Session,
    ) -> Result<(), Self::Error> {
        match std::str::from_utf8(data) {
            Ok(unparsed_cmd) if unparsed_cmd.trim().starts_with("git-receive-pack") => {
                // TODO: proper parsing for things like quotes, escape characters, etc.

                let mut cmd = unparsed_cmd.trim().split_whitespace();
                // ignore `git-receive-pack`
                let _ = cmd.next();

                let Some(repo_path) = cmd.next() else {
                    return Err(SshHandlerErr::UnknownCommand);
                };

                // right now, since we aren't doing any proper parsing, assert that atleast *some* bare minimal assumptions hold true.
                assert!(repo_path.starts_with("'"));
                assert!(repo_path.ends_with("'"));

                // ignore the first and last ASCII chars, we know they're quotes (and hence 1 byte) because of the assert above.
                let repo_path = &repo_path[1..repo_path.len() - 1];
                let repo_path = format!(
                    "{}/repositories/{}",
                    self.data_dir.clone().into_os_string().to_string_lossy(),
                    repo_path
                );
                println!("Writing to {repo_path}");

                session.channel_success(channel_id).unwrap();
                // TODO: error handling.
                let repo = git2::Repository::open_bare(repo_path).unwrap();

                // dunno buffer size yet.
                let (tx_owned, mut rx) = tokio::sync::mpsc::channel(10);
                // Write out the pack. This has to be done on a seperate thread as `PackWriter`
                // is not Send, and hence any `.await` won't work.
                tokio::task::spawn_blocking({
                    let channels = std::sync::Arc::clone(&self.channels);
                    move || {
                        let mut buff: Vec<PackWorkerMsg> = Vec::new();
                        let db = repo.odb().unwrap();
                        let mut channels = channels.blocking_lock();
                        let Some(channel) = channels.get_mut(&channel_id) else {
                            panic!("SshHandler: receive-pack: Channel Not Found");
                            // return Err(SshHandlerErr::ChannelNotFound);
                        };
                        std::mem::drop(channels);
                        loop {
                            let mut pack_writer = db.packwriter().unwrap();
                            rx.blocking_recv_many(&mut buff, 10);
                            println!("got item!");
                            for i in &buff {
                                match i {
                                    PackWorkerMsg::Data(crypto_vec) => {
                                        println!("writing!");
                                        pack_writer.write_all(&crypto_vec).unwrap_or_else(|e| {
                                            panic!("Failed to write pack to disk: \n\t {e}")
                                        });
                                        println!("done!");
                                    }
                                    PackWorkerMsg::Eof => {
                                        println!("got eof, commiting...");
                                        pack_writer.commit().unwrap();
                                    }
                                }
                            }
                            buff.clear();
                        }
                    }
                });

                let channels = std::sync::Arc::clone(&self.channels);
                {
                    {
                        let tx = &tx_owned;
                        let mut channels = channels.lock().await;
                        let Some(channel) = channels.get_mut(&channel_id) else {
                            panic!("SshHandler: receive-pack: Channel Not Found");
                            // return Err(SshHandlerErr::ChannelNotFound);
                        };
                        loop {
                            println!("waiting for a message");
                            match channel.wait().await {
                                Some(msg) => match dbg!(msg) {
                                    russh::ChannelMsg::Exec {
                                        want_reply: true,
                                        command,
                                    } => {
                                        if command == unparsed_cmd.as_bytes() {
                                            println!("succ");
                                            session.channel_success(channel_id).unwrap();
                                        } else {
                                            session.channel_failure(channel_id).unwrap();
                                        }
                                    }
                                    russh::ChannelMsg::Data { data } => {
                                        println!("sent item!");
                                        tx.send(PackWorkerMsg::Data(data)).await.unwrap();
                                    }
                                    russh::ChannelMsg::Eof => {}
                                    russh::ChannelMsg::Close => {
                                        tx.send(PackWorkerMsg::Eof).await.unwrap();
                                        match channel.close().await {
                                            Ok(_) => {}
                                            Err(russh::Error::SendError) => {
                                                eprintln!(
                                            "Connection terminated unexpectedly, cleaning up..."
                                        )
                                            }
                                            e => {
                                                eprintln!(
                                                    "Unexpected error while closing channel: {e:?}"
                                                );
                                            }
                                        }
                                        channels.remove_entry(&channel_id);
                                        std::mem::drop(tx_owned);
                                        break;
                                    }
                                    msg => {
                                        eprintln!(
                                            "Got unexpected message while reading pack:\n\t{msg:?}"
                                        )
                                    }
                                },
                                _ => todo!(),
                            }
                        }
                    }
                };
                return Ok(());
            }
            _ => return Err(SshHandlerErr::UnknownCommand),
        }
    }
}
