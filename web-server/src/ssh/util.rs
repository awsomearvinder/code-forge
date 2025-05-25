use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;

// read from `src` into `dst` until `pat` is found
// or `src` is at EOF.
pub async fn read_until_bytes<Reader>(
    src: &mut BufReader<Reader>,
    dst: &mut Vec<u8>,
    pat: &[u8],
) -> Result<usize, tokio::io::Error>
where
    Reader: AsyncRead + Unpin,
{
    // empty case, just read out the entire buffer.
    if pat.is_empty() {
        return src.read_to_end(dst).await;
    }

    let mut bytes_read = 0;
    let mut buff = vec![0; pat.len()];
    // read until we find the first byte matching `pat`, and then
    // check if following bytes also match. If not, re-loop.
    loop {
        bytes_read += src.read_until(pat[0], dst).await?;
        if pat.len() == 1 {
            return Ok(bytes_read);
        }
        bytes_read += src.read_exact(&mut buff).await?;
        dst.extend(&buff);
        if buff == pat[1..] {
            return Ok(bytes_read);
        } else {
            dst.extend(&buff);
            buff.clear();
            continue;
        }
    }
}
