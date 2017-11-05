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

use ffi;

/// A `Message` represents all the information passed in an MQTT PUBLISH 
/// packet.
/// This is the primary data transfer mechanism.
#[derive(Debug)]
pub struct Message {
	pub cmsg: ffi::MQTTAsync_message,
	pub topic: CString,
	pub payload: Vec<u8>
}

impl Message {
	/// Creates a new message.
	/// 
	/// # Arguments
	/// 
	/// * `topic` The topic on which the message is published.
	/// * `payload` The binary payload of the message
	pub fn new<V>(topic: &str, payload: V) -> Message
		where V: Into<Vec<u8>>
	{
		let msg = Message {
			cmsg: ffi::MQTTAsync_message::default(),
			topic: CString::new(topic).unwrap(),
			payload: payload.into(),
		};
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

/////////////////////////////////////////////////////////////////////////////

/// Builder to create a new Message
#[derive(Debug)]
pub struct MessageBuilder
{
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
	pub fn topic(&mut self, topic: &str) -> &mut MessageBuilder {
		self.topic = topic.to_string();
		self
	}

	/// Sets the payload for the message
	///
	/// # Arguments
	///
	/// `payload` The binary payload of the message
	pub fn payload<V>(&mut self, payload: V) -> &mut MessageBuilder
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
	pub fn qos(&mut self, qos: i32) -> &mut MessageBuilder {
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
	pub fn retained(&mut self, retained: bool) -> &mut MessageBuilder {
		self.retained = retained;
		self
	}

	/// Finalize the builder to create the message.
	pub fn finalize(&self) -> Message {
		let mut msg = Message {
			cmsg: ffi::MQTTAsync_message::default(),
			topic: CString::new(self.topic.clone()).unwrap(),
			payload: self.payload.clone(),
		};
		msg.cmsg.qos = self.qos;
		msg.cmsg.retained = if self.retained { 1 } else { 0 };
		Message::fixup(msg)
	}
}



