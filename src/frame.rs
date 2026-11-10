//! Framing: `u32` big-endian length prefix + postcard body of at most
//! [`MAX_FRAME`] bytes.

use std::fmt;

use serde::{de::DeserializeOwned, Serialize};

use crate::consts::MAX_FRAME;

/// Size of the length prefix in bytes.
pub const HEADER_LEN: usize = 4;

#[derive(Debug, PartialEq, Eq)]
pub enum FrameError {
	/// The body is larger than [`MAX_FRAME`] (when encoding, or as announced by a header).
	TooLarge(usize),
	/// The buffer does not hold a full frame yet; `needed` is the total frame size if known.
	Incomplete { needed: Option<usize> },
	/// The body is not a valid postcard encoding of the expected type.
	Decode(postcard::Error),
	/// The value could not be encoded.
	Encode(postcard::Error),
}

impl fmt::Display for FrameError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::TooLarge(len) => write!(f, "frame body of {len} bytes exceeds {MAX_FRAME}"),
			Self::Incomplete { needed: Some(n) } => write!(f, "incomplete frame, need {n} bytes"),
			Self::Incomplete { needed: None } => write!(f, "incomplete frame header"),
			Self::Decode(e) => write!(f, "invalid frame body: {e}"),
			Self::Encode(e) => write!(f, "cannot encode frame body: {e}"),
		}
	}
}

impl std::error::Error for FrameError {}

/// Encodes `msg` as one frame.
pub fn encode<T: Serialize>(msg: &T) -> Result<Vec<u8>, FrameError> {
	let body = postcard::to_stdvec(msg).map_err(FrameError::Encode)?;
	if body.len() > MAX_FRAME {
		return Err(FrameError::TooLarge(body.len()));
	}
	let mut frame = Vec::with_capacity(HEADER_LEN + body.len());
	frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
	frame.extend_from_slice(&body);
	Ok(frame)
}

/// Body length announced by a frame header, checked against [`MAX_FRAME`]. Lets a reader know
/// how many bytes to read before calling [`decode`].
pub fn body_len(header: [u8; HEADER_LEN]) -> Result<usize, FrameError> {
	let len = u32::from_be_bytes(header) as usize;
	if len > MAX_FRAME {
		return Err(FrameError::TooLarge(len));
	}
	Ok(len)
}

/// Decodes the first frame in `buf`. Returns the message and the number of bytes consumed.
pub fn decode<T: DeserializeOwned>(buf: &[u8]) -> Result<(T, usize), FrameError> {
	let header: [u8; HEADER_LEN] = buf
		.get(..HEADER_LEN)
		.ok_or(FrameError::Incomplete { needed: None })?
		.try_into()
		.expect("slice has HEADER_LEN bytes");
	let total = HEADER_LEN + body_len(header)?;
	let body = buf
		.get(HEADER_LEN..total)
		.ok_or(FrameError::Incomplete { needed: Some(total) })?;
	let msg = postcard::from_bytes(body).map_err(FrameError::Decode)?;
	Ok((msg, total))
}

#[cfg(test)]
mod tests {
	use std::net::SocketAddr;

	use super::*;
	use crate::messages::*;

	fn round_trip<T: Serialize + DeserializeOwned + PartialEq + fmt::Debug>(msg: T) {
		let frame = encode(&msg).unwrap();
		assert_eq!(
			body_len(frame[..HEADER_LEN].try_into().unwrap()).unwrap(),
			frame.len() - HEADER_LEN
		);
		let (decoded, used) = decode::<T>(&frame).unwrap();
		assert_eq!(decoded, msg);
		assert_eq!(used, frame.len());
	}

	fn record() -> Record {
		Record {
			id: [1; 32],
			seq: 1_700_000_000_000,
			relay_url: Some("https://relay.example.com".into()),
			direct: vec![
				"203.0.113.7:7843".parse::<SocketAddr>().unwrap(),
				"[2001:db8::1]:7843".parse().unwrap(),
			],
			namespaces: vec!["secret-holder/1".into()],
			expires_in_secs: 600,
		}
	}

	#[test]
	fn registry_messages_round_trip() {
		round_trip(RegistryRequest::Register {
			seq: 5,
			relay_url: Some("https://relay.example.com".into()),
			direct: record().direct,
			namespaces: vec!["secret-holder/1".into()],
			ttl_secs: 600,
		});
		round_trip(RegistryRequest::Unregister);
		round_trip(RegistryRequest::Lookup { id: [9; 32] });
		round_trip(RegistryRequest::Discover {
			namespace: "secret-holder/1".into(),
			cursor: Some(40),
			limit: 100,
		});
		round_trip(RegistryResponse::Registered { ttl_secs: 600 });
		round_trip(RegistryResponse::Ok);
		round_trip(RegistryResponse::Found(Some(record())));
		round_trip(RegistryResponse::Found(None));
		round_trip(RegistryResponse::Discovered {
			records: vec![record(), record()],
			cursor: None,
		});
		round_trip(RegistryResponse::Err(RegistryError::StaleSeq));
	}

	#[test]
	fn job_and_secret_messages_round_trip() {
		round_trip(JobHello {
			target_job: vec![0, 1, 2],
			protocol: "echo".into(),
			source_job: None,
		});
		round_trip(JobHello {
			target_job: vec![3],
			protocol: "x".into(),
			source_job: Some(vec![4, 5]),
		});
		round_trip(JobHelloReply::Ok);
		round_trip(JobHelloReply::Err(JobError::NotRunningHere));
		round_trip(SecretRequest { generation: 7 });
		round_trip(SecretResponse::Secret { generation: 7, secret: [8; 32] });
		round_trip(SecretResponse::Err(SecretError::Forbidden));
	}

	#[test]
	fn assignment_messages_round_trip() {
		round_trip(AssignmentNotice { block: 230_571, jobs: vec![vec![0, 1, 2], vec![3]] });
		round_trip(AssignmentNotice { block: 1, jobs: vec![] });
		round_trip(AssignmentNoticeReply::Ok);
		round_trip(AssignmentNoticeReply::Err(AssignmentNoticeError::Forbidden));
	}

	#[test]
	fn manage_messages_round_trip() {
		round_trip(ManageRequest { sent_at_ms: 1_762_700_000_000, command: ManageCommand::Status });
		round_trip(ManageRequest { sent_at_ms: 1, command: ManageCommand::Reboot { force: true } });
		round_trip(ManageRequest {
			sent_at_ms: 1,
			command: ManageCommand::RestartApp { force: false },
		});
		round_trip(ManageReply::Status(ProcessorStatus {
			app_version: "1.2.3 (456)".into(),
			os: "Android 16".into(),
			battery_percent: Some(87),
			charging: None,
			busy: true,
			uptime_secs: 3_600,
		}));
		round_trip(ManageReply::Accepted);
		round_trip(ManageReply::Err(ManageError::Busy));
	}

	#[test]
	fn gossip_round_trip_and_signing_payload() {
		round_trip(SignedGossip {
			author: [1; 32],
			seq: 2,
			sent_at_ms: 3,
			payload: vec![4; 10],
			sig: [5; 64],
		});

		let topic = crate::hash::topic_id("health/v1");
		let payload = SignedGossip::signing_payload(&topic, &[1; 32], 2, 3, b"hi");
		assert!(payload.starts_with(crate::TAG_GOSSIP));
		assert_eq!(&payload[crate::TAG_GOSSIP.len()..][..32], &topic);
		// Any change of a signed field changes what is signed.
		assert_ne!(payload, SignedGossip::signing_payload(&topic, &[1; 32], 3, 3, b"hi"));
	}

	#[test]
	fn enum_variants_keep_their_wire_index() {
		// postcard tags enum variants by index: reordering variants would break the wire format.
		assert_eq!(postcard::to_stdvec(&RegistryRequest::Unregister).unwrap(), vec![1]);
		assert_eq!(postcard::to_stdvec(&RegistryResponse::Ok).unwrap(), vec![1]);
		assert_eq!(postcard::to_stdvec(&JobHelloReply::Ok).unwrap(), vec![0]);
		assert_eq!(postcard::to_stdvec(&JobError::Forbidden).unwrap(), vec![3]);
		assert_eq!(postcard::to_stdvec(&SecretError::Busy).unwrap(), vec![2]);
	}

	#[test]
	fn too_large_and_incomplete_frames() {
		let too_big = SignedGossip {
			author: [0; 32],
			seq: 0,
			sent_at_ms: 0,
			payload: vec![0; MAX_FRAME],
			sig: [0; 64],
		};
		assert!(matches!(encode(&too_big), Err(FrameError::TooLarge(_))));
		assert!(matches!(
			body_len(((MAX_FRAME + 1) as u32).to_be_bytes()),
			Err(FrameError::TooLarge(_))
		));

		let frame = encode(&SecretRequest { generation: 1 }).unwrap();
		assert_eq!(
			decode::<SecretRequest>(&frame[..2]),
			Err(FrameError::Incomplete { needed: None })
		);
		assert_eq!(
			decode::<SecretRequest>(&frame[..frame.len() - 1]),
			Err(FrameError::Incomplete { needed: Some(frame.len()) })
		);
	}

	#[test]
	fn decode_reads_only_the_first_frame() {
		let mut buf = encode(&SecretRequest { generation: 1 }).unwrap();
		let first_len = buf.len();
		buf.extend(encode(&SecretRequest { generation: 2 }).unwrap());
		let (first, used) = decode::<SecretRequest>(&buf).unwrap();
		assert_eq!((first.generation, used), (1, first_len));
		let (second, _) = decode::<SecretRequest>(&buf[used..]).unwrap();
		assert_eq!(second.generation, 2);
	}

	#[test]
	fn garbage_body_is_rejected() {
		let mut frame = vec![0, 0, 0, 2];
		frame.extend([0xff, 0xff]);
		assert!(matches!(decode::<RegistryRequest>(&frame), Err(FrameError::Decode(_))));
	}
}
