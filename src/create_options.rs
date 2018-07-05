// create_options.rs
//
// The set of options for creating an MQTT client.
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017-2018 Frank Pagliughi <fpagliughi@mindspring.com>
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

/// The type of persistence for the client
pub enum PersistenceType {
	/// Data and messages are persisted to a local file (default)
	File,
	/// No persistence is used.
	None,
	/// A user-defined persistence provided by the application.
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

/// The options for creating an MQTT client.
/// This can be constructed using a 
/// [CreateOptionsBuilder](struct.CreateOptionsBuilder.html).
#[derive(Debug)]
pub struct CreateOptions {
	/// The underlying C option structure
	pub(crate) copts: ffi::MQTTAsync_createOptions,
	/// The URI for the MQTT broker.
	pub(crate) server_uri: String,
	/// The unique name for the client.
	/// This can be left empty for the server to assign a random name.
	pub(crate) client_id: String,
	/// The type of persistence used by the client.
	pub(crate) persistence: PersistenceType,
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
	/// Constructs a set of CreatieOptions with reasonable defaults.
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

/// Builder to construct client creation options.
///
/// # Examples
///
/// ```
/// extern crate paho_mqtt as mqtt;
/// 
/// let opts = mqtt::CreateOptionsBuilder::new()
///                    .server_uri("tcp://localhost:1883")
///                    .client_id("client1")
///                    .finalize();
/// 
/// let cli = mqtt::AsyncClient::new(opts).unwrap();
/// ```

pub struct CreateOptionsBuilder {
	copts: ffi::MQTTAsync_createOptions,
	server_uri: String,
	client_id: String,
	persistence: PersistenceType,
}

impl CreateOptionsBuilder {
	/// Constructs a builder with default options.
	pub fn new() -> CreateOptionsBuilder {
		CreateOptionsBuilder {
			copts: ffi::MQTTAsync_createOptions::default(),
			server_uri: "".to_string(),
			client_id: "".to_string(),
			persistence: PersistenceType::File,
		}
	}

	/// Sets the the URI to the MQTT broker.
	/// Alternately, the application can specify multiple servers via the 
	/// connect options.
	///
	/// # Arguments
	///
	/// `server_uri` The URI string to specify the server in the form 
	///              _protocol://host:port_, where the protocol can be
	///              _tcp_ or _ssl_, and the host can be an IP address
	///              or domain name.
	pub fn server_uri<S>(mut self, server_uri: S) -> CreateOptionsBuilder
			where S: Into<String> {
		self.server_uri = server_uri.into();
		self
	}

	/// Sets the client identifier string that is sent to the server.
	/// The client ID is a unique name to identify the client to the server,
	/// which can be used if the client desires the server to hold state 
	/// about the session. If the client requests a clean sesstion, this can
	/// be an empty string.
	/// 
	/// The broker is required to honor a client ID of up to 23 bytes, but 
	/// could honor longer ones, depending on the broker.
	/// 
	/// Note that if this is an empty string, the clean session parameter 
	/// *must* be set to _true_.
	/// 
	/// # Arguments
	///
	/// `client_id` A UTF-8 string identifying the client to the server.
	///
	pub fn client_id<S>(mut self, client_id: S) -> CreateOptionsBuilder
			where S: Into<String> {
		self.client_id = client_id.into();
		self
	}

	/// Sets the type of persistence used by the client.
	/// The default is for the library to automatically use file persistence,
	/// although this can be turned off by specify `None` for a more 
	/// performant, though possibly less reliable system.
	///
	/// # Arguments
	///
	/// `persist` The type of persistence to use.
	///
	pub fn persistence(mut self, persist: PersistenceType) -> CreateOptionsBuilder {
		self.persistence = persist;
		self
	}

	/// Sets a user-defined persistence store.
	/// This sets the persistence to use a custom one defined by the 
	/// application. This can be anything that implements the 
	/// `ClientPersistence` trait.
	///
	/// # Arguments
	///
	/// `persist` An application-defined custom persistence store.
	///
	pub fn user_persistence<T>(mut self, persistence: T) -> CreateOptionsBuilder 
			where T: ClientPersistence + 'static
	{
		let persistence: Box<Box<ClientPersistence>> = Box::new(Box::new(persistence));
		self.persistence = PersistenceType::User(persistence);
		self
	}

	/// Sets the maximum number of messages that can be buffered for delivery
	/// when the client is off-line.
	/// The client has limited support for bufferering messages when the 
	/// client is temporarily disconnected. This specifies the maximum number
	/// of messages that can be buffered.
	///
	/// # Arguments
	///
	/// `n` The maximum number of messages that can be buffered. Setting this
	///     to zero disables off-line buffering.
	///
	pub fn max_buffered_messages(mut self, n: i32) -> CreateOptionsBuilder {
		self.copts.maxBufferedMessages = n;
		self.copts.sendWhileDisconnected = if n == 0 { 0 } else { 1 };
		self
	}

	/// Constructs a set of create options from the builder information.
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
    use std::os::raw::{c_char};

    // The identifier for the create options structure
    const STRUCT_ID: [c_char; 4] = [ b'M' as c_char, b'Q' as c_char, b'C' as c_char, b'O' as c_char];

	// Rust options should be the same as the C options
	#[test]
	fn test_default() {
		let opts = CreateOptions::default();
		// Get default C options for comparison
		let copts = ffi::MQTTAsync_createOptions::default();

		// First, make sure C options valid
        assert_eq!(STRUCT_ID, copts.struct_id);
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

		assert_eq!(STRUCT_ID, opts.copts.struct_id);
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

		assert_eq!(STRUCT_ID, opts.copts.struct_id);
		assert_eq!(0, opts.copts.struct_version);	// Currently supported version
		assert_eq!(copts.sendWhileDisconnected, opts.copts.sendWhileDisconnected);
		assert_eq!(copts.maxBufferedMessages, opts.copts.maxBufferedMessages);

		assert_eq!(HOST, &opts.server_uri);
		assert_eq!(ID, &opts.client_id);
		//assert_eq!(PersistenceType::File, opts.persistence);
	}

	#[test]
	fn test_default_builder() {
		let opts = CreateOptionsBuilder::new().finalize();
		let copts = ffi::MQTTAsync_createOptions::default();

		// First, make sure C options valid
		assert_eq!(STRUCT_ID, copts.struct_id);
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

		assert_eq!(STRUCT_ID, opts.copts.struct_id);
		assert_eq!(0, opts.copts.struct_version);	// Currently supported version

		assert_eq!(HOST, &opts.server_uri);
		assert_eq!(ID, &opts.client_id);
		//assert_eq!(PersistenceType::File, opts.persistence);
		assert!(0 != opts.copts.sendWhileDisconnected);
		assert_eq!(MAX_BUF_MSGS, opts.copts.maxBufferedMessages);
	}
}


