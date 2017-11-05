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
use std::ffi::{CString, IntoStringError};
use std::string::{FromUtf8Error};
use std::os::raw::{c_void};

use ffi;

/// The options for the Last Will and Testament (LWT)
#[derive(Debug)]
pub struct WillOptions {
	pub opts: ffi::MQTTAsync_willOptions,
	topic: CString,
	payload: Vec<u8>,
}

impl WillOptions {
	pub fn new() -> WillOptions {
		let opts = WillOptions {
			opts: ffi::MQTTAsync_willOptions::default(),
			topic: CString::new("").unwrap(),
			payload: Vec::new(),
		};
		WillOptions::fixup(opts)
	}

	pub fn from_message<V>(topic: &str, payload: V) -> WillOptions
		where V: Into<Vec<u8>>
	{
		let opts = WillOptions {
			opts: ffi::MQTTAsync_willOptions::default(),
			topic: CString::new(topic).unwrap(),
			payload: payload.into(),
		};
		WillOptions::fixup(opts)
	}

	// Updates the C struct from the cached topic and payload vars
	fn fixup(mut opts: WillOptions) -> WillOptions {
		opts.opts.topicName = opts.topic.as_ptr();
		opts.opts.payload.data = opts.payload.as_ptr() as *const c_void;
		opts.opts.payload.len = opts.payload.len() as i32;
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
		self.opts.qos
	}

	/// Gets the 'retained' flag for the message.
	pub fn get_retained(&self) -> bool {
		self.opts.retained != 0
	}
}

impl Clone for WillOptions {
	fn clone(&self) -> WillOptions {
		let will = WillOptions {
			opts: self.opts.clone(),
			topic: self.topic.clone(),
			payload: self.payload.clone(),
		};
		WillOptions::fixup(will)
	}
}

#[derive(Debug,Clone)]
pub struct WillOptionsBuilder { 
	topic: CString,
	payload: Vec<u8>,
	retained: bool,
	qos: i32,
}

impl WillOptionsBuilder {
	pub fn new() -> WillOptionsBuilder {
		let copts = ffi::MQTTAsync_willOptions::default();

		WillOptionsBuilder {
			topic: CString::new("").unwrap(),
			payload: Vec::new(),
			retained: copts.retained != 0,
			qos: copts.qos,
		}
	}

	pub fn topic(&mut self, topic: &str) -> &mut WillOptionsBuilder {
		self.topic = CString::new(topic).unwrap();
		self
	}

	pub fn payload<V>(&mut self, payload: V) -> &mut WillOptionsBuilder
		where V: Into<Vec<u8>>
	{
		self.payload = payload.into();
		self
	}

	pub fn retained(&mut self, retained: bool) -> &mut WillOptionsBuilder {
		self.retained = retained;
		self
	}

	pub fn qos(&mut self, qos: i32) -> &mut WillOptionsBuilder {
		self.qos = qos;
		self
	}

	pub fn finalize(&self) -> WillOptions {
		let opts = WillOptions {
			opts: ffi::MQTTAsync_willOptions::default(),
			topic: self.topic.clone(),
			payload: self.payload.clone(),
		};
		WillOptions::fixup(opts)
	}
}

