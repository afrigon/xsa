use bitcode::{DecodeOwned, Encode};
use quinn::{RecvStream, SendStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{DisconnectReason, FrameStream, Received};

pub(super) struct StreamPump {
    connection: quinn::Connection,
    send: FrameStream<SendStream>,
    receive: FrameStream<RecvStream>,
}

impl StreamPump {
    pub fn new(connection: quinn::Connection, send: SendStream, receive: RecvStream) -> StreamPump {
        StreamPump {
            connection,
            send: FrameStream::new(send),
            receive: FrameStream::new(receive),
        }
    }

    pub async fn run<Outgoing: Encode, Incoming: DecodeOwned>(
        self,
        mut outgoing: UnboundedReceiver<Received<Outgoing>>,
        incoming: UnboundedSender<Received<Incoming>>,
    ) {
        let StreamPump {
            connection,
            mut send,
            mut receive,
        } = self;
        let writer = async {
            loop {
                match outgoing.recv().await {
                    Some(Received::Frame(frame)) => {
                        if send.write(&frame).await.is_err() {
                            return;
                        }
                    }
                    Some(Received::Disconnected { reason }) => return StreamPump::close(&connection, &reason),
                    None => return StreamPump::close(&connection, &DisconnectReason::Lost),
                }
            }
        };
        let reader = async {
            let reason = loop {
                match receive.read().await {
                    Ok(Some(frame)) => {
                        if incoming.send(Received::Frame(frame)).is_err() {
                            return;
                        }
                    }
                    Ok(None) => break DisconnectReason::from_connection_error(&connection.closed().await),
                    Err(err) => break StreamPump::read_failed(&connection, &err),
                }
            };
            let _ = incoming.send(Received::Disconnected { reason });
        };

        tokio::select! {
            () = writer => {}
            () = reader => {}
        }
    }

    fn read_failed(connection: &quinn::Connection, err: &anyhow::Error) -> DisconnectReason {
        if let Some(closed) = connection.close_reason() {
            return DisconnectReason::from_connection_error(&closed);
        }

        let reason = DisconnectReason::ProtocolViolation {
            error: format!("{err:#}"),
        };
        tracing::warn!("{} sent an invalid frame: {err:#}", connection.remote_address());
        StreamPump::close(connection, &reason);

        reason
    }

    fn close(connection: &quinn::Connection, reason: &DisconnectReason) {
        connection.close(reason.code(), reason.message());
    }
}
