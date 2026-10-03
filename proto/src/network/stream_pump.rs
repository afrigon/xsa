use bitcode::{DecodeOwned, Encode};
use quinn::{RecvStream, SendStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::frame_stream::FrameStream;

pub(super) struct StreamPump {
    send: FrameStream<SendStream>,
    receive: FrameStream<RecvStream>,
}

impl StreamPump {
    pub fn new(send: SendStream, receive: RecvStream) -> StreamPump {
        StreamPump {
            send: FrameStream::new(send),
            receive: FrameStream::new(receive),
        }
    }

    pub async fn run<Outgoing: Encode, Incoming: DecodeOwned>(
        self,
        mut outgoing: UnboundedReceiver<Outgoing>,
        incoming: UnboundedSender<Incoming>,
    ) -> anyhow::Result<()> {
        let StreamPump { mut send, mut receive } = self;
        let writer = async {
            while let Some(value) = outgoing.recv().await {
                send.write(&value).await?;
            }

            send.into_inner().finish()?;

            anyhow::Ok(())
        };
        let reader = async {
            while let Some(value) = receive.read().await? {
                if incoming.send(value).is_err() {
                    break;
                }
            }

            anyhow::Ok(())
        };

        tokio::select! {
            result = writer => result,
            result = reader => result,
        }
    }
}
