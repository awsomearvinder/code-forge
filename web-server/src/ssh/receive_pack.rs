use std::path::{Path, PathBuf};

use futures::AsyncWriteExt;
use gix::bstr::ByteSlice;
use russh::Channel;
use tokio::io::BufReader;
use tokio_util::compat::TokioAsyncWriteCompatExt as _;

use super::SshHandlerErr;

async fn reference_discovery(
    channel: &mut Channel<russh::server::Msg>,
    repo: gix::Repository,
) -> Result<gix::Repository, SshHandlerErr> {
    let mut lines = vec![];
    let writer = channel.make_writer();
    let mut writer = gix_packetline::Writer::new(writer.compat_write()).text_mode();
    // we need to make sure we drop everything that touches the `repo`
    // when we turn the `repo` into a ThreadSafeRepository again. Do work
    // that references the `repo` in it's own scope.
    {
        let refs = repo.references().unwrap();
        let branches = refs.local_branches().unwrap();
        for branch in branches {
            let Some(branch) = branch.ok() else {
                continue;
            };
            // TODO: This should not assume unicode.
            let pkt_line = format!(
                "{} {}\0",
                branch.id(),
                branch.name().as_bstr().to_str_lossy().into_owned(),
            )
            .into_bytes();
            lines.extend(pkt_line);
        }
    }
    let repo = repo.into_sync();
    writer.write_all(&lines).await.unwrap();
    gix_packetline::encode::flush_to_write(writer.inner_mut())
        .await
        .unwrap();
    writer.flush().await.unwrap();

    Ok(repo.to_thread_local())
}

// TODO: need to abstract this to not rely on ssh.
pub async fn git_receive_pack<P: AsRef<Path>>(
    cmd: Vec<u8>,
    data_dir: P,
    channel: &mut Channel<russh::server::Msg>,
) -> Result<(), SshHandlerErr> {
    const CMD_NAME: &[u8] = b"git-receive-pack";
    if !cmd.trim().starts_with(CMD_NAME) {
        return Err(SshHandlerErr::UnexpectedCommand);
    }

    // TODO: proper parsing for things like quotes, escape characters, etc.
    let args = cmd.trim()[CMD_NAME.len()..].trim();

    let repo_path = args;
    // right now, since we aren't doing any proper parsing, assert that atleast *some* bare minimal assumptions hold true.
    assert!(repo_path.starts_with(b"'"));
    assert!(repo_path.ends_with(b"'"));

    let repo_name = &repo_path[1..repo_path.len() - 1];

    println!("receiving pack!");
    // TODO: Make sure paths are safe. It's a security issue otherwise.
    // I'm going to assume the client will send it over UTF-8 always on the network.
    let mut repo_path = PathBuf::from("./");
    repo_path.push(data_dir);
    repo_path.push("repositories");
    repo_path.push(String::from("./") + &String::from_utf8_lossy(repo_name));

    let mut repo = gix::open(repo_path.clone())
        .map_err(SshHandlerErr::FailedToOpenRepo)
        .unwrap();
    loop {
        let msg = match dbg!(channel.wait().await) {
            Some(msg) => msg,
            _ => todo! {},
        };
        match msg {
            russh::ChannelMsg::Exec {
                want_reply: true,
                command,
            } => {
                if cmd != command {
                    return Err(SshHandlerErr::UnexpectedCommand);
                }
                repo = reference_discovery(channel, repo).await.unwrap();
                let reader = channel.make_reader();
                let mut reader = BufReader::new(reader);
                let mut client_data = vec![];
                const FLUSH_PKT: &[u8; 4] = b"0000";
                // TODO: error handling.
                crate::ssh::util::read_until_bytes(&mut reader, &mut client_data, FLUSH_PKT)
                    .await
                    .unwrap();

                // Client early exited without doing anything.
                if client_data == FLUSH_PKT {
                    return Ok(());
                }
                dbg!(client_data);
            }
            russh::ChannelMsg::Data { data: _ } => {
                println!("sent item!");
            }
            russh::ChannelMsg::Eof => {
                println!("eof")
            }
            russh::ChannelMsg::Close => {
                break;
            }
            msg => {
                eprintln!("Got unexpected message while reading pack:\n\t{msg:?}")
            }
        }
    }
    Ok(())
}
