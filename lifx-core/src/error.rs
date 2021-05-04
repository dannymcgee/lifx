use std::io;
use thiserror::Error;

/// Various message encoding/decoding errors
#[derive(Error, Debug)]
pub enum Error {
	/// This error means we were unable to parse a raw message because its type is unknown.
	///
	/// LIFX devices are known to send messages that are not officially documented, so this error
	/// type does not necessarily represent a bug.
	#[error("unknown message type: `{0}`")]
	UnknownMessageType(u16),
	/// This error means one of the message fields contains an invalid or unsupported value.
	#[error("protocol error: `{0}`")]
	ProtocolError(String),

	#[error("i/o error")]
	Io(#[from] io::Error),
}

impl From<std::convert::Infallible> for Error {
	fn from(_: std::convert::Infallible) -> Self {
		unreachable!()
	}
}
