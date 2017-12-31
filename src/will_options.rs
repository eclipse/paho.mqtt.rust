// will_options.rs
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017 Frank Pagliughi <fpagliughi@mindspring.com>
 *
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v1.0
 * and Eclipse Distribution License v1.0 which accompany this distribution.
 *
 * The Eclipse Public License is available at
 *    http://www.eclipse.org/legal/epl-v10.html
 * and the Eclipse Distribution License is available at
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Frank Pagliughi - initial implementation and documentation
 *******************************************************************************/

//use std::slice;
use std::ptr;
use std::ffi::{CString, IntoStringError};
use std::string::{FromUtf8Error};
use std::os::raw::{c_void};

use ffi;

use message::Message;

// TODO: We probably don't need the will options... at least not for the
// public API. This is simply a message. So the public API could be:
// 
//   let lwt = Message::new(...);
//   let opts = ConnectOptionsBuilder::new().will_message(lwt).finalize();
//

/// The options for the Last Will and Testament (LWT).
/// This defines a message that is registered with the the server at the time
/// of connection. Then if the connection is lost unexpectedly, the message
/// is published by the server. 
///
#[derive(Debug)]
pub struct WillOptions {
	pub(crate) copts: ffi::MQTTAsync_willOptions,
	topic: CString,
	payload: Vec<u8>,
}

impl WillOptions {
	pub fn new() -> WillOptions {
		WillOptions::default()
	}

	pub fn from_message<V>(topic: &str, payload: V) -> WillOptions
		where V: Into<Vec<u8>>
	{
		let opts = WillOptions {
			copts: ffi::MQTTAsync_willOptions::default(),
			topic: CString::new(topic).unwrap(),
			payload: payload.into(),
		};
		WillOptions::fixup(opts)
	}

	// Updates the C struct from the cached topic and payload vars
	fn fixup(mut opts: WillOptions) -> WillOptions {
		opts.copts.topicName = if opts.topic.as_bytes().len() != 0 {
			opts.topic.as_ptr() 
		} 
		else {
			ptr::null()
		};
		opts.copts.payload.data = if opts.payload.len() != 0 {
			opts.payload.as_ptr() as *const c_void
		}
		else {
			ptr::null()
		};
		opts.copts.payload.len = opts.payload.len() as i32;
		opts
	}

	/// Gets the topic string for the LWT
	fn get_topic(&self) -> Result<String, IntoStringError> {
		self.topic.clone().into_string()
	}

	/// Gets the payload of the LWT
	pub fn get_payload(&self) -> &Vec<u8> {
		&self.payload
	}

	/// Gets the payload of the message as a string.
	/// Note that this clones the payload.
	pub fn get_payload_str(&self) -> Result<String, FromUtf8Error> {
		String::from_utf8(self.payload.clone())
	}

	/// Returns the Quality of Service (QOS) for the message.
	pub fn get_qos(&self) -> i32 {
		self.copts.qos
	}

	/// Gets the 'retained' flag for the message.
	pub fn get_retained(&self) -> bool {
		self.copts.retained != 0
	}
}

impl Default for WillOptions {
	fn default() -> WillOptions {
		let opts = WillOptions {
			copts: ffi::MQTTAsync_willOptions::default(),
			topic: CString::new("").unwrap(),
			payload: Vec::new(),
		};
		WillOptions::fixup(opts)
	}
}

impl Clone for WillOptions {
	fn clone(&self) -> WillOptions {
		let will = WillOptions {
			copts: self.copts.clone(),
			topic: self.topic.clone(),
			payload: self.payload.clone(),
		};
		WillOptions::fixup(will)
	}
}

impl From<Message> for WillOptions {
	/// Create `WillOptions` from a `Message`
	fn from(msg: Message) -> Self {
		let mut will = WillOptions {
			copts: ffi::MQTTAsync_willOptions::default(),
			topic: msg.topic,
			payload: msg.payload,
		};
		will.copts.qos = msg.cmsg.qos;
		will.copts.retained = msg.cmsg.retained;
		WillOptions::fixup(will)
	}
}

/////////////////////////////////////////////////////////////////////////////
//									Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
	//use std::ffi::{CStr};
	use message::MessageBuilder;

	const STRUCT_ID: &'static [i8] = &[ 'M' as i8, 'Q' as i8, 'T' as i8, 'W' as i8];
	const STRUCT_VERSION: i32 = 1;

	// These should differ from defaults.
	const TOPIC: &'static str = "test";
	const PAYLOAD: &'static [u8] = b"Hello world";
	const QOS: i32 = 2;
	const RETAINED: bool = true;

	// By convention our defaults should match the defaults of the C library
	#[test]
	fn test_default() {
		let opts = WillOptions::default();
		// Get default C options for comparison
		let copts = ffi::MQTTAsync_willOptions::default();

		// First, make sure C options valid
		assert_eq!(STRUCT_ID, copts.struct_id);
		assert_eq!(STRUCT_VERSION, copts.struct_version);
		assert_eq!(ptr::null(), copts.message);

		assert_eq!(copts.struct_id, opts.copts.struct_id);
		assert_eq!(copts.struct_version, opts.copts.struct_version);
		assert_eq!(copts.topicName, opts.copts.topicName);
		assert_eq!(copts.retained, opts.copts.retained);
		assert_eq!(copts.qos, opts.copts.qos);
		assert_eq!(copts.payload.len, opts.copts.payload.len);
		assert_eq!(copts.payload.data, opts.copts.payload.data);

		assert_eq!(&[] as &[u8], opts.topic.as_bytes());
		assert_eq!(&[] as &[u8], opts.payload.as_slice());
	}

	// Test creating will options from a message
	#[test]
	fn test_from_message() {
		let msg = MessageBuilder::new()
						.topic(TOPIC)
						.payload(PAYLOAD)
						.qos(QOS)
						.retained(RETAINED)
						.finalize();

		let opts = WillOptions::from(msg);

		assert_eq!(TOPIC, opts.topic.to_str().unwrap());
		assert_eq!(PAYLOAD, opts.payload.as_slice());

		assert_eq!(opts.payload.len() as i32, opts.copts.payload.len);
		assert_eq!(opts.payload.as_ptr() as *const c_void, opts.copts.payload.data);

		assert_eq!(QOS, opts.copts.qos);
		assert!(opts.copts.retained != 0);
	}

	// Make sure assignment works properly
	// This primarily ensures that C pointers stay fixed to cached values,
	// and/or that the cached buffers don't move due to assignment.
	#[test]
	fn test_assign() {
		let msg = MessageBuilder::new()
						.topic(TOPIC)
						.payload(PAYLOAD)
						.qos(QOS)
						.retained(RETAINED)
						.finalize();

		let org_opts = WillOptions::from(msg);
		let opts = org_opts;

		assert_eq!(TOPIC, opts.topic.to_str().unwrap());
		assert_eq!(PAYLOAD, opts.payload.as_slice());

		assert_eq!(opts.payload.len() as i32, opts.copts.payload.len);
		assert_eq!(opts.payload.as_ptr() as *const c_void, opts.copts.payload.data);

		assert_eq!(QOS, opts.copts.qos);
		assert!(opts.copts.retained != 0);
	}

	// Test that a clone works properly.
	// This ensures that the cached values are cloned and that the C pointers
	// in the new object point to those clones.
	#[test]
	fn test_clone() {
		let opts = {
			let msg = MessageBuilder::new()
							.topic(TOPIC)
							.payload(PAYLOAD)
							.qos(QOS)
							.retained(RETAINED)
							.finalize();

			let org_opts = WillOptions::from(msg);
			org_opts.clone()
		};

		assert_eq!(TOPIC, opts.topic.to_str().unwrap());
		assert_eq!(PAYLOAD, opts.payload.as_slice());

		assert_eq!(opts.payload.len() as i32, opts.copts.payload.len);
		assert_eq!(opts.payload.as_ptr() as *const c_void, opts.copts.payload.data);

		assert_eq!(QOS, opts.copts.qos);
		assert!(opts.copts.retained != 0);
	}
}
