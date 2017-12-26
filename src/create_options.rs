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

use std::fmt;

use ffi;
use client_persistence::ClientPersistence;

/*
Remember the C constants (c_uint)
  MQTTCLIENT_PERSISTENCE_DEFAULT = 0
  MQTTCLIENT_PERSISTENCE_NONE    = 1
  MQTTCLIENT_PERSISTENCE_USER    = 2
*/

pub enum PersistenceType {
	File,
	None,
	User(Box<Box<ClientPersistence>>),
}

impl fmt::Debug for PersistenceType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			PersistenceType::File => write!(f, "File"),
			PersistenceType::None => write!(f, "None"),
			PersistenceType::User(_) => write!(f, "User"),
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
//							Create Options
/////////////////////////////////////////////////////////////////////////////

/**
 * The options for creating an MQTT client.
 */
#[derive(Debug)]
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

/////////////////////////////////////////////////////////////////////////////
//								Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
	use super::*;

	// Rust options should be the same as the C options
	#[test]
	fn test_default() {
		let opts = CreateOptions::default();
		let copts = ffi::MQTTAsync_createOptions::default();

		// First, make sure C options valid
		assert_eq!([ 'M' as i8, 'Q' as i8, 'C' as i8, 'O' as i8], copts.struct_id);
		assert_eq!(0, copts.struct_version);	// Currently supported version

		assert_eq!(copts.struct_id, opts.copts.struct_id);
		assert_eq!(copts.struct_version, opts.copts.struct_version);
		assert_eq!(copts.sendWhileDisconnected, opts.copts.sendWhileDisconnected);
		assert_eq!(copts.maxBufferedMessages, opts.copts.maxBufferedMessages);

		assert_eq!("", &opts.server_uri);
		assert_eq!("", &opts.client_id);
		//assert_eq!(PersistenceType::File, opts.persistence);
	}

	#[test]
	fn test_from_string() {
		const HOST: &'static str = "localhost";
		let opts = CreateOptions::from(HOST);
		let copts = ffi::MQTTAsync_createOptions::default();

		assert_eq!([ 'M' as i8, 'Q' as i8, 'C' as i8, 'O' as i8], opts.copts.struct_id);
		assert_eq!(0, opts.copts.struct_version);	// Currently supported version
		assert_eq!(copts.sendWhileDisconnected, opts.copts.sendWhileDisconnected);
		assert_eq!(copts.maxBufferedMessages, opts.copts.maxBufferedMessages);

		assert_eq!(HOST, &opts.server_uri);
		assert_eq!("", &opts.client_id);
		//assert_eq!(PersistenceType::File, opts.persistence);
	}


	#[test]
	fn test_from_tuple() {
		const HOST: &'static str = "localhost";
		const ID: &'static str = "bubba";
		let opts = CreateOptions::from((HOST,ID));
		let copts = ffi::MQTTAsync_createOptions::default();

		assert_eq!([ 'M' as i8, 'Q' as i8, 'C' as i8, 'O' as i8], opts.copts.struct_id);
		assert_eq!(0, opts.copts.struct_version);	// Currently supported version
		assert_eq!(copts.sendWhileDisconnected, opts.copts.sendWhileDisconnected);
		assert_eq!(copts.maxBufferedMessages, opts.copts.maxBufferedMessages);

		assert_eq!(HOST, &opts.server_uri);
		assert_eq!(ID, &opts.client_id);
		//assert_eq!(PersistenceType::File, opts.persistence);
	}

	#[test]
	fn test_defaultbuilder() {
		let opts = CreateOptionsBuilder::new().finalize();
		let copts = ffi::MQTTAsync_createOptions::default();

		// First, make sure C options valid
		assert_eq!([ 'M' as i8, 'Q' as i8, 'C' as i8, 'O' as i8], copts.struct_id);
		assert_eq!(0, copts.struct_version);	// Currently supported version

		assert_eq!(copts.struct_id, opts.copts.struct_id);
		assert_eq!(copts.struct_version, opts.copts.struct_version);
		assert_eq!(copts.sendWhileDisconnected, opts.copts.sendWhileDisconnected);
		assert_eq!(copts.maxBufferedMessages, opts.copts.maxBufferedMessages);

		assert_eq!("", &opts.server_uri);
		assert_eq!("", &opts.client_id);
		//assert_eq!(PersistenceType::File, opts.persistence);
	}

	#[test]
	fn test_builder() {
		const HOST: &'static str = "localhost";
		const ID: &'static str = "bubba";
		const MAX_BUF_MSGS: i32 = 100;

		let opts = CreateOptionsBuilder::new()
						.server_uri(HOST)
						.client_id(ID)
						// TODO: Test persistence
						.max_buffered_messages(MAX_BUF_MSGS)
						.finalize();

		assert_eq!([ 'M' as i8, 'Q' as i8, 'C' as i8, 'O' as i8], opts.copts.struct_id);
		assert_eq!(0, opts.copts.struct_version);	// Currently supported version

		assert_eq!(HOST, &opts.server_uri);
		assert_eq!(ID, &opts.client_id);
		//assert_eq!(PersistenceType::File, opts.persistence);
		assert!(0 != opts.copts.sendWhileDisconnected);
		assert_eq!(MAX_BUF_MSGS, opts.copts.maxBufferedMessages);
	}
}


