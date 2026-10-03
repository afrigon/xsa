use anyhow::ensure;
use bitcode::{DecodeOwned, Encode};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const FRAME_LENGTH_BYTES: usize = 4;
const MAXIMUM_FRAME_BYTES: usize = 16 * 1024 * 1024;

pub struct FrameStream<Stream> {
    stream: Stream,
}

impl<Stream> FrameStream<Stream> {
    pub fn new(stream: Stream) -> FrameStream<Stream> {
        FrameStream { stream }
    }

    pub fn into_inner(self) -> Stream {
        self.stream
    }
}

impl<Stream: AsyncWrite + Unpin> FrameStream<Stream> {
    pub async fn write<T: Encode>(&mut self, value: &T) -> anyhow::Result<()> {
        let payload = bitcode::encode(value);
        ensure!(
            payload.len() <= MAXIMUM_FRAME_BYTES,
            "a {} byte frame is too large",
            payload.len()
        );

        self.stream.write_all(&(payload.len() as u32).to_le_bytes()).await?;
        self.stream.write_all(&payload).await?;
        self.stream.flush().await?;

        Ok(())
    }
}

impl<Stream: AsyncRead + Unpin> FrameStream<Stream> {
    // `None` when the stream ends cleanly between frames.
    pub async fn read<T: DecodeOwned>(&mut self) -> anyhow::Result<Option<T>> {
        let mut length = [0; FRAME_LENGTH_BYTES];
        let first_read = self.stream.read(&mut length).await?;

        if first_read == 0 {
            return Ok(None);
        }

        self.stream.read_exact(&mut length[first_read..]).await?;
        let length = u32::from_le_bytes(length) as usize;
        ensure!(length <= MAXIMUM_FRAME_BYTES, "a {length} byte frame is too large");

        let mut payload = vec![0; length];
        self.stream.read_exact(&mut payload).await?;

        Ok(Some(bitcode::decode(&payload)?))
    }
}
