use std::fmt;

use quinn::{ConnectionError, VarInt};

// Code 0 is what QUIC sends when a connection is dropped without a reason.
const LOST_CODE: VarInt = VarInt::from_u32(0);
const LEFT_CODE: VarInt = VarInt::from_u32(1);
const SERVER_STOPPED_CODE: VarInt = VarInt::from_u32(2);
const PROTOCOL_VIOLATION_CODE: VarInt = VarInt::from_u32(3);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisconnectReason {
    Left,
    ServerStopped,
    ProtocolViolation { error: String },
    Lost,
}

impl DisconnectReason {
    pub(super) fn from_connection_error(error: &ConnectionError) -> DisconnectReason {
        let ConnectionError::ApplicationClosed(close) = error else {
            return DisconnectReason::Lost;
        };

        if close.error_code == LEFT_CODE {
            DisconnectReason::Left
        } else if close.error_code == SERVER_STOPPED_CODE {
            DisconnectReason::ServerStopped
        } else if close.error_code == PROTOCOL_VIOLATION_CODE {
            DisconnectReason::ProtocolViolation {
                error: String::from_utf8_lossy(&close.reason).into_owned(),
            }
        } else {
            DisconnectReason::Lost
        }
    }

    pub(super) fn code(&self) -> VarInt {
        match self {
            DisconnectReason::Left => LEFT_CODE,
            DisconnectReason::ServerStopped => SERVER_STOPPED_CODE,
            DisconnectReason::ProtocolViolation { .. } => PROTOCOL_VIOLATION_CODE,
            DisconnectReason::Lost => LOST_CODE,
        }
    }

    pub(super) fn message(&self) -> &[u8] {
        match self {
            DisconnectReason::ProtocolViolation { error } => error.as_bytes(),
            DisconnectReason::Left | DisconnectReason::ServerStopped | DisconnectReason::Lost => &[],
        }
    }
}

impl fmt::Display for DisconnectReason {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DisconnectReason::Left => write!(formatter, "left"),
            DisconnectReason::ServerStopped => write!(formatter, "the server stopped"),
            DisconnectReason::ProtocolViolation { error } => write!(formatter, "protocol violation: {error}"),
            DisconnectReason::Lost => write!(formatter, "the connection was lost"),
        }
    }
}

impl std::error::Error for DisconnectReason {}

#[cfg(test)]
mod tests {
    use quinn::ApplicationClose;

    use super::*;

    #[test]
    fn every_reason_survives_a_close() {
        let reasons = [
            DisconnectReason::Left,
            DisconnectReason::ServerStopped,
            DisconnectReason::ProtocolViolation {
                error: "a 20 byte frame is too large".to_string(),
            },
            DisconnectReason::Lost,
        ];

        for reason in reasons {
            let closed = ConnectionError::ApplicationClosed(ApplicationClose {
                error_code: reason.code(),
                reason: reason.message().to_vec().into(),
            });
            assert_eq!(DisconnectReason::from_connection_error(&closed), reason);
        }
    }

    #[test]
    fn a_connection_ending_without_a_close_is_lost() {
        assert_eq!(
            DisconnectReason::from_connection_error(&ConnectionError::TimedOut),
            DisconnectReason::Lost
        );
    }
}
