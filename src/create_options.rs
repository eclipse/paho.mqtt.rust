// create_options.rs
//
// The set of options for creating an MQTT client.
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

use ffi;
use client_persistence::ClientPersistence;

/*
pub const MQTTCLIENT_PERSISTENCE_DEFAULT: ::std::os::raw::c_uint = 0;
pub const MQTTCLIENT_PERSISTENCE_NONE:    ::std::os::raw::c_uint = 1;
pub const MQTTCLIENT_PERSISTENCE_USER:    ::std::os::raw::c_uint = 2;
*/

//#[derive(PartialEq)]
pub enum PersistenceType {
	File,
	None,
	User(Box<Box<ClientPersistence>>),
}

/////////////////////////////////////////////////////////////////////////////
///

/**
 * The options for creating an MQTT client.
 */
//#[derive(Debug)]
pub struct CreateOptions {
	pub copts: ffi::MQTTAsync_createOptions,
	pub server_uri: String,
	pub client_id: String,
	pub persistence: PersistenceType,
}

impl CreateOptions {
	pub fn new() -> CreateOptions {
		CreateOptions::default()
	}
}

impl<'a> From<&'a str> for CreateOptions {
	fn from(server_uri: &'a str) -> Self {
		let mut opts = CreateOptions::default();
		opts.server_uri = server_uri.to_string();
		opts
	}
}

impl<'a, 'b> From<(&'a str, &'b str)> for CreateOptions {
	fn from((server_uri, client_id): (&'a str, &'b str)) -> Self {
		let mut opts = CreateOptions::default();
		opts.server_uri = server_uri.to_string();
		opts.client_id = client_id.to_string();
		opts
	}
}

impl Default for CreateOptions {
	fn default() -> CreateOptions {
		CreateOptions {
			copts: ffi::MQTTAsync_createOptions::default(),
			server_uri: "".to_string(),
			client_id: "".to_string(),
			persistence: PersistenceType::File,
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
//								Builder
/////////////////////////////////////////////////////////////////////////////

pub struct CreateOptionsBuilder {
	copts: ffi::MQTTAsync_createOptions,
	server_uri: String,
	client_id: String,
	persistence: PersistenceType,
}

impl CreateOptionsBuilder {
	pub fn new() -> CreateOptionsBuilder {
		CreateOptionsBuilder {
			copts: ffi::MQTTAsync_createOptions::default(),
			server_uri: "".to_string(),
			client_id: "".to_string(),
			persistence: PersistenceType::File,
		}
	}

	pub fn server_uri<S>(mut self, server_uri: S) -> CreateOptionsBuilder
			where S: Into<String> {
		self.server_uri = server_uri.into();
		self
	}

	pub fn client_id<S>(mut self, client_id: S) -> CreateOptionsBuilder
			where S: Into<String> {
		self.client_id = client_id.into();
		self
	}

	pub fn persistence(mut self, persist: PersistenceType) -> CreateOptionsBuilder {
		self.persistence = persist;
		self
	}

	pub fn user_persistence<T>(mut self, persistence: T) -> CreateOptionsBuilder 
			where T: ClientPersistence + 'static
	{
		let persistence: Box<Box<ClientPersistence>> = Box::new(Box::new(persistence));
		self.persistence = PersistenceType::User(persistence);
		self
	}

	pub fn max_buffered_messages(mut self, n: i32) -> CreateOptionsBuilder {
		self.copts.maxBufferedMessages = n;
		self.copts.sendWhileDisconnected = if n == 0 { 0 } else { 1 };
		self
	}

	pub fn finalize(self) -> CreateOptions {
		CreateOptions {
			copts: self.copts,
			server_uri: self.server_uri,
			client_id: self.client_id,
			persistence: self.persistence,
		}
	}
}

