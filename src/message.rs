// message.rs
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

use std::slice;
use std::ffi::{CString, IntoStringError};
use std::string::{FromUtf8Error};
use std::os::raw::{c_void};
use std::convert::From;

use ffi;

/// A `Message` represents all the information passed in an MQTT PUBLISH 
/// packet.
/// This is the primary data transfer mechanism.
#[derive(Debug)]
pub struct Message {
	pub cmsg: ffi::MQTTAsync_message,
	pub topic: CString,
	payload: Vec<u8>
}

impl Message {
	/// Creates a new message.
	/// 
	/// # Arguments
	/// 
	/// * `topic` The topic on which the message is published.
	/// * `payload` The binary payload of the message
	/// * `qos` The quality of service for message delivery (0, 1, or 2)
	pub fn new<V>(topic: &str, payload: V, qos: i32) -> Message
		where V: Into<Vec<u8>>
	{
		let mut msg = Message {
			cmsg: ffi::MQTTAsync_message::default(),
			topic: CString::new(topic).unwrap(),
			payload: payload.into(),
		};
		msg.cmsg.qos = qos;
		Message::fixup(msg)
	}

	/// Creates a new message that will be retained by the broker.
	/// This creates a message with the 'retained' flag set.
	/// 
	/// # Arguments
	/// 
	/// * `topic` The topic on which the message is published.
	/// * `payload` The binary payload of the message
	/// * `qos` The quality of service for message delivery (0, 1, or 2)
	pub fn new_retained<V>(topic: &str, payload: V, qos: i32) -> Message
		where V: Into<Vec<u8>>
	{
		let mut msg = Message {
			cmsg: ffi::MQTTAsync_message::default(),
			topic: CString::new(topic).unwrap(),
			payload: payload.into(),
		};
		msg.cmsg.qos = qos;
		msg.cmsg.retained = 1;	// true
		Message::fixup(msg)
	}

	/// Creates a new message from C language components.
	/// 
	/// # Arguments
	/// 
	/// * `topic` The topic on which the message is published.
	/// * `msg` The message struct from the C library
	pub unsafe fn from_c_parts(topic: CString, cmsg: &ffi::MQTTAsync_message) -> Message {
		let len = cmsg.payloadlen as usize;
		let payload =  slice::from_raw_parts(cmsg.payload as *mut u8, len);

		let msg = Message {
			cmsg: cmsg.clone(),
			topic,
			payload: payload.to_vec(),
		};
		Message::fixup(msg)
	}

	// Ensures that the underlying C struct points to cached values
	fn fixup(mut msg: Message) -> Message {
		msg.cmsg.payload = msg.payload.as_mut_ptr() as *mut c_void;
		msg.cmsg.payloadlen = msg.payload.len() as i32;
		msg
	}

	/// Gets the topic for the message.
	/// Note that this copies the topic.
	pub fn get_topic(&self) -> Result<String, IntoStringError> {
		self.topic.clone().into_string()
	}

	/// Gets the payload of the message.
	/// This returns the payload as a binary vector.
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
		self.cmsg.qos
	}

	/// Gets the 'retained' flag for the message.
	pub fn get_retained(&self) -> bool {
		self.cmsg.retained != 0
	}
}

impl Default for Message {
	fn default() -> Message {
		let msg = Message {
			cmsg: ffi::MQTTAsync_message::default(),
			topic: CString::new("").unwrap(),
			payload: Vec::new(),
		};
		Message::fixup(msg)
	}
}

impl Clone for Message {
	fn clone(&self) -> Message {
		let msg = Message {
			cmsg: self.cmsg.clone(),
			topic: self.topic.clone(),
			payload: self.payload.clone(),
		};
		Message::fixup(msg)
	}
}

impl<'a, 'b> From<(&'a str, &'b [u8])> for Message {
	fn from((topic, payload): (&'a str, &'b [u8])) -> Self {
		let msg = Message {
			cmsg: ffi::MQTTAsync_message::default(),
			topic: CString::new(topic).unwrap(),
			payload: payload.to_vec(),
		};
		Message::fixup(msg)
	}
}

impl<'a, 'b> From<(&'a str, &'b [u8], i32, bool)> for Message {
	fn from((topic, payload, qos, retained): (&'a str, &'b [u8], i32, bool)) -> Self {
		let mut msg = Message {
			cmsg: ffi::MQTTAsync_message::default(),
			topic: CString::new(topic).unwrap(),
			payload: payload.to_vec(),
		};
		msg.cmsg.qos = qos;
		msg.cmsg.retained = if retained { 1 } else { 0 };
		Message::fixup(msg)
	}
}

/////////////////////////////////////////////////////////////////////////////

/// Builder to create a new Message
#[derive(Debug)]
pub struct MessageBuilder {
	topic: String,
	payload: Vec<u8>,
	qos: i32,
	retained: bool,
}

impl MessageBuilder
{
	/// Create a new message builder.
	pub fn new() -> MessageBuilder {
		MessageBuilder {
			topic: String::new(),
			payload: Vec::new(),
			qos: 0,
			retained: false,
		}
	}

	/// Sets the topic for the message
	///
	/// # Arguments
	/// 
	/// `topic` The topic on which the message should be published.
	pub fn topic(mut self, topic: &str) -> MessageBuilder {
		self.topic = topic.to_string();
		self
	}

	/// Sets the payload for the message
	///
	/// # Arguments
	///
	/// `payload` The binary payload of the message
	pub fn payload<V>(mut self, payload: V) -> MessageBuilder
		where V: Into<Vec<u8>>
	{
		self.payload = payload.into();
		self
	}

	/// Sets the Quality of Service for the message.
	///
	/// # Arguments
	///
	/// `qos` The quality of service for the message.
	pub fn qos(mut self, qos: i32) -> MessageBuilder {
		self.qos = qos;
		self
	}

	/// Sets whether or not the published message should be retained by the 
	/// broker.
	///
	/// # Arguments
	/// 
	/// `retained` Set true if the message should be retained by the broker,
	///			   false if not.
	pub fn retained(mut self, retained: bool) -> MessageBuilder {
		self.retained = retained;
		self
	}

	/// Finalize the builder to create the message.
	pub fn finalize(self) -> Message {
		let mut msg = Message {
			cmsg: ffi::MQTTAsync_message::default(),
			topic: CString::new(self.topic).unwrap(),
			payload: self.payload,
		};
		msg.cmsg.qos = self.qos;
		msg.cmsg.retained = if self.retained { 1 } else { 0 };
		Message::fixup(msg)
	}
}

/////////////////////////////////////////////////////////////////////////////
//								Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
	use super::*;
	//use std::ffi::{CStr};
	use std::os::raw::{c_char};

	#[test]
	fn test_new() {
		const TOPIC: &'static str = "test";
		const PAYLOAD: &'static [u8] = b"Hello world";
		const QOS: i32 = 2;

		let msg = Message::new(TOPIC, PAYLOAD, QOS);
		assert_eq!([ 'M' as c_char, 'Q' as i8, 'T' as i8, 'M' as i8 ], msg.cmsg.struct_id);
		assert_eq!(0, msg.cmsg.struct_version);

		assert_eq!(TOPIC, msg.topic.to_str().unwrap());
		assert_eq!(PAYLOAD, msg.payload.as_slice());

		assert_eq!(msg.payload.len() as i32, msg.cmsg.payloadlen);
		assert_eq!(msg.payload.as_ptr() as *mut c_void, msg.cmsg.payload);

		assert_eq!(QOS, msg.cmsg.qos);
		assert!(msg.cmsg.retained == 0);
	}

	#[test]
	fn test_from_tuple() {
		const TOPIC: &'static str = "test";
		const PAYLOAD: &'static [u8] = b"Hello world";
		const QOS: i32 = 2;
		const RETAINED: bool = true;

		let msg = Message::from((TOPIC, PAYLOAD, QOS, RETAINED));

		// The topic is only kept in the Rust struct as a CString
		assert_eq!(TOPIC, msg.topic.to_str().unwrap());
		assert_eq!(PAYLOAD, msg.payload.as_slice());

		assert_eq!(msg.payload.len() as i32, msg.cmsg.payloadlen);
		assert_eq!(msg.payload.as_ptr() as *mut c_void, msg.cmsg.payload);

		assert_eq!(QOS, msg.cmsg.qos);
		assert!(msg.cmsg.retained != 0);
	}

	#[test]
	fn test_builder_default() {
		let msg = MessageBuilder::new().finalize();
		let cmsg = ffi::MQTTAsync_message::default();

		assert_eq!([ 'M' as c_char, 'Q' as i8, 'T' as i8, 'M' as i8 ], cmsg.struct_id);
		assert_eq!(0, cmsg.struct_version);

		assert_eq!(cmsg.struct_id, msg.cmsg.struct_id);
		assert_eq!(cmsg.struct_version, msg.cmsg.struct_version);

		assert_eq!(0, msg.topic.as_bytes().len());
		assert_eq!(&[] as &[u8], msg.topic.as_bytes());
		assert_eq!(vec![] as Vec<u8>, msg.payload);
	}

	#[test]
	fn test_builder_topic() {
		const TOPIC: &'static str = "test";

		let msg = MessageBuilder::new()
					.topic(TOPIC).finalize();

		// The topic is only kept in the Rust struct as a CString
		assert_eq!(TOPIC, msg.topic.to_str().unwrap());
	}

	#[test]
	fn test_builder_payload() {
		const PAYLOAD: &'static [u8] = b"Hello world";

		let msg = MessageBuilder::new()
					.payload(PAYLOAD).finalize();

		assert_eq!(PAYLOAD, msg.payload.as_slice());

		assert_eq!(msg.payload.len() as i32, msg.cmsg.payloadlen);
		assert_eq!(msg.payload.as_ptr() as *mut c_void, msg.cmsg.payload);
	}

	#[test]
	fn test_builder_qos() {
		const QOS: i32 = 2;

		let msg = MessageBuilder::new()
					.qos(QOS).finalize();

		assert_eq!(QOS, msg.cmsg.qos);
	}

	#[test]
	fn test_builder_retained() {
		let msg = MessageBuilder::new()
					.retained(false).finalize();
		assert!(msg.cmsg.retained == 0);

		let msg = MessageBuilder::new()
					.retained(true).finalize();
		assert!(msg.cmsg.retained != 0);
	}

	#[test]
	fn test_copy() {
		const TOPIC: &'static str = "test";
		const PAYLOAD: &'static [u8] = b"Hello world";
		const QOS: i32 = 2;
		const RETAINED: bool = true;

		let org_msg = MessageBuilder::new()
						.topic(TOPIC)
						.payload(PAYLOAD)
						.qos(QOS)
						.retained(RETAINED)
						.finalize();

		let msg = org_msg;

		// The topic is only kept in the Rust struct as a CString
		assert_eq!(TOPIC, msg.topic.to_str().unwrap());
		assert_eq!(PAYLOAD, msg.payload.as_slice());

		assert_eq!(msg.payload.len() as i32, msg.cmsg.payloadlen);
		assert_eq!(msg.payload.as_ptr() as *mut c_void, msg.cmsg.payload);

		assert_eq!(QOS, msg.cmsg.qos);
		assert!(msg.cmsg.retained != 0);
	}

	#[test]
	fn test_clone() {
		const TOPIC: &'static str = "test";
		const PAYLOAD: &'static [u8] = b"Hello world";
		const QOS: i32 = 2;
		const RETAINED: bool = true;

		let msg = {
			// Make sure the original goes out of scope before testing
			let org_msg = MessageBuilder::new()
							.topic(TOPIC)
							.payload(PAYLOAD)
							.qos(QOS)
							.retained(RETAINED)
							.finalize();
			org_msg.clone()
		};

		assert_eq!(TOPIC, msg.topic.to_str().unwrap());
		assert_eq!(PAYLOAD, msg.payload.as_slice());

		assert_eq!(msg.payload.len() as i32, msg.cmsg.payloadlen);
		assert_eq!(msg.payload.as_ptr() as *mut c_void, msg.cmsg.payload);

		assert_eq!(QOS, msg.cmsg.qos);
		assert!(msg.cmsg.retained != 0);
	}

}

