use anyhow::ensure;
use bitcode::{DecodeOwned, Encode};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const FRAME_LENGTH_BYTES: usize = 4;
const MAXIMUM_FRAME_BYTES: usize = 16 * 1024 * 1024;

pub async fn write_frame<T: Encode>(stream: &mut (impl AsyncWrite + Unpin), value: &T) -> anyhow::Result<()> {
    let payload = bitcode::encode(value);
    ensure!(
        payload.len() <= MAXIMUM_FRAME_BYTES,
        "a {} byte frame is too large",
        payload.len()
    );
    stream.write_all(&(payload.len() as u32).to_le_bytes()).await?;
    stream.write_all(&payload).await?;
    stream.flush().await?;
    Ok(())
}

// `None` when the stream ends cleanly between frames.
pub async fn read_frame<T: DecodeOwned>(stream: &mut (impl AsyncRead + Unpin)) -> anyhow::Result<Option<T>> {
    let mut length = [0; FRAME_LENGTH_BYTES];
    let first_read = stream.read(&mut length).await?;
    if first_read == 0 {
        return Ok(None);
    }
    stream.read_exact(&mut length[first_read..]).await?;
    let length = u32::from_le_bytes(length) as usize;
    ensure!(length <= MAXIMUM_FRAME_BYTES, "a {length} byte frame is too large");
    let mut payload = vec![0; length];
    stream.read_exact(&mut payload).await?;
    Ok(Some(bitcode::decode(&payload)?))
}
