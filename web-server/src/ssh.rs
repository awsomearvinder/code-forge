use gix::bstr::ByteSlice;
use receive_pack::git_receive_pack;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use russh::{Channel, ChannelId};

mod util;

mod receive_pack;

pub struct SshServer {
    data_dir: PathBuf,
}
impl SshServer {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }
}

impl russh::server::Server for SshServer {
    type Handler = GitSshHandler;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        Self::Handler::new(self.data_dir.clone())
    }
}

#[derive(Debug)]
struct ChannelData {
    params: Vec<String>,
    channel: Channel<russh::server::Msg>,
}
pub struct GitSshHandler {
    channel_lookup_table: std::sync::Arc<tokio::sync::Mutex<HashMap<ChannelId, ChannelData>>>,
    data_dir: PathBuf,
}

impl GitSshHandler {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            channel_lookup_table: Default::default(),
            data_dir,
        }
    }
    async fn add_channel(&mut self, channel_id: ChannelId, channel: Channel<russh::server::Msg>) {
        let mut ch = self.channel_lookup_table.lock().await;
        assert!(ch
            .insert(
                channel_id,
                ChannelData {
                    params: vec![],
                    channel,
                },
            )
            .is_none());
    }
    async fn remove_channel(&mut self, channel_id: ChannelId) {
        let mut ch = self.channel_lookup_table.lock().await;
        ch.remove_entry(&channel_id).unwrap();
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum SshHandlerErr {
    ChannelNotFound,
    Disconnect,
    FailedToOpenRepo(gix::open::Error),
    UnexpectedCommand,
    UnknownCommand,
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

impl russh::server::Handler for GitSshHandler {
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

    async fn env_request(
        &mut self,
        channel: ChannelId,
        variable_name: &str,
        variable_value: &str,
        session: &mut russh::server::Session,
    ) -> Result<(), Self::Error> {
        println!("env request...");
        let mut channel_lookup_table = self.channel_lookup_table.lock().await;
        let channel_data = channel_lookup_table
            .get_mut(&channel)
            .ok_or(SshHandlerErr::ChannelNotFound)?;
        println!("{variable_name} is wanting to be set to {variable_value}");
        if variable_name == "GIT_PROTOCOL" {
            channel_data.params.clear();
            channel_data
                .params
                .extend(variable_value.split(':').map(String::from));
        }
        session.handle().channel_success(channel).await.unwrap();
        Ok(())
    }

    async fn channel_open_session(
        &mut self,
        channel: russh::Channel<russh::server::Msg>,
        _session: &mut russh::server::Session,
    ) -> Result<bool, Self::Error> {
        println!("channel_open_session...");
        self.add_channel(channel.id(), channel).await;
        Ok(true)
    }
    async fn channel_close(
        &mut self,
        channel: russh::ChannelId,
        _session: &mut russh::server::Session,
    ) -> Result<(), Self::Error> {
        println!("removing channel...");
        self.remove_channel(channel).await;
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel_id: russh::ChannelId,
        cmd: &[u8],
        _session: &mut russh::server::Session,
    ) -> Result<(), Self::Error> {
        println!("{}", String::from_utf8_lossy(cmd));
        const VALID_CMDS: &[&str] = &["git-receive-pack", "git-upload-pack"];
        let Some(cmd_name) = VALID_CMDS
            .iter()
            .copied()
            .find(|potential_cmd| cmd.trim().starts_with(potential_cmd.as_bytes()))
        else {
            return Err(SshHandlerErr::UnexpectedCommand);
        };
        let data_dir = self.data_dir.clone();
        let lookup_table = Arc::clone(&self.channel_lookup_table);
        let cmd = Vec::from(cmd);

        tokio::spawn(async move {
            let mut lookup_table = lookup_table.lock().await;
            let Some(ChannelData { params: _, channel }) = lookup_table.get_mut(&channel_id) else {
                panic!("Failed to get channel with channel id {channel_id}");
            };
            match cmd_name {
                "git-receive-pack" => git_receive_pack(cmd, data_dir, channel)
                    .await
                    .expect("git recv pack failed..."),
                _ => panic!("unknown command {cmd_name}"),
            }
            channel.close().await.unwrap();
        });
        Ok(())
    }
}
