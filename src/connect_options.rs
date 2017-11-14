// connect_options.rs
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
use std::ptr;
use std::time::Duration;
use std::ffi::{CString};
use std::os::raw::{/*c_void, c_char,*/ c_int};
use will_options::WillOptions;
use ssl_options::SslOptions;
use string_collection::StringCollection;

pub const MQTT_VERSION_DEFAULT: u32	= ffi::MQTTVERSION_DEFAULT;
pub const MQTT_VERSION_3_1: u32		= ffi::MQTTVERSION_3_1;
pub const MQTT_VERSION_3_1_1: u32	= ffi::MQTTVERSION_3_1_1;

/////////////////////////////////////////////////////////////////////////////
// Connections

/// The collection of options for connecting to a broker.
#[derive(Debug)]
pub struct ConnectOptions {
	pub copts: ffi::MQTTAsync_connectOptions,
	will: Option<Box<WillOptions>>,
	ssl: Option<Box<SslOptions>>,
	user_name: CString,
	password: CString,
	server_uris: StringCollection,
}

impl ConnectOptions {
	/// Creates a new, default set of connect options
	pub fn new() -> ConnectOptions {
		ConnectOptions::default()
	}

	// Fixes up the underlying C struct to point to our cached values.
	fn fixup(mut opts: ConnectOptions) -> ConnectOptions {
		opts.copts.will = if let Some(ref mut will_opts) = opts.will {
			&mut will_opts.opts
		}
		else {
			ptr::null_mut()
		};

		opts.copts.ssl = if let Some(ref mut ssl_opts) = opts.ssl {
			&mut ssl_opts.copts
		}
		else {
			ptr::null_mut()
		};

		opts.copts.username = if opts.user_name.as_bytes().len() != 0 {
			opts.user_name.as_ptr()
		}
		else {
			ptr::null()
		};

		opts.copts.password = if opts.password.as_bytes().len() != 0 {
			opts.password.as_ptr()
		}
		else {
			ptr::null()
		};
		//opts.copts.password = opts.password.as_ptr();

		let n = opts.server_uris.len();
		if n != 0 {
			opts.copts.serverURIs = opts.server_uris.as_c_arr_ptr();
			opts.copts.serverURIcount = n as c_int;
		}
		else {
			opts.copts.serverURIs = ptr::null();
			opts.copts.serverURIcount = 0;
		}

		opts
	}
/*
	pub fn to_c_struct(&mut self) -> ffi::MQTTAsync_connectOptions {
		let mut opts = self.opts.clone();

		opts.will = if let Some(ref mut will_opts) = opts.will {
			&mut will_opts.opts
		}
		else {
			ptr::null_mut()
		};

		//opts.ssl = &mut self.ssl.opts; // as *mut _;
		opts
	}
*/
	pub fn set_clean_session(&mut self, clean: bool) {
		self.copts.cleansession = if clean { 1 } else { 0 }
	}

	pub fn get_clean_session(&self) -> bool {
		self.copts.cleansession != 0
	}
}

impl Default for ConnectOptions {
	fn default() -> ConnectOptions {
		let opts = ConnectOptions {
			copts: ffi::MQTTAsync_connectOptions::default(),
			will: None,
			ssl: None,
			user_name: CString::new("").unwrap(),
			password: CString::new("").unwrap(),
			server_uris: StringCollection::default(),
		};
		ConnectOptions::fixup(opts)
	}
}

impl Clone for ConnectOptions {
    fn clone(&self) -> ConnectOptions { 
		let opts = ConnectOptions {
			copts: self.copts.clone(),
			will: self.will.clone(),
			ssl: self.ssl.clone(),
			user_name: self.user_name.clone(),
			password: self.password.clone(),
			server_uris: self.server_uris.clone(),
		};
		ConnectOptions::fixup(opts)
	}
}

/////////////////////////////////////////////////////////////////////////////
//								Builder
/////////////////////////////////////////////////////////////////////////////

pub struct ConnectOptionsBuilder {
	copts: ffi::MQTTAsync_connectOptions,
	will: Option<WillOptions>,
	ssl: Option<SslOptions>,
	user_name: String,
	password: String,
	server_uris: StringCollection,
}

impl ConnectOptionsBuilder {
	pub fn new() -> ConnectOptionsBuilder {
		ConnectOptionsBuilder {
			copts: ffi::MQTTAsync_connectOptions::default(),
			will: None,
			ssl: None,
			user_name: "".to_string(),
			password: "".to_string(),
			server_uris: StringCollection::default(),
		}
	}

	/// Sets the keep alive interval for the client session.
	///
	/// # Arguments
	/// 
	/// `keep_alive_interval` The maximum time that should pass without 
	/// 					  communication between the client and server.
	///						  This has a resolution in seconds.
	pub fn keep_alive_interval(&mut self, keep_alive_interval: Duration) -> &mut ConnectOptionsBuilder {
		let secs = keep_alive_interval.as_secs();
		self.copts.keepAliveInterval = if secs == 0 { 1 } else { secs as i32 };
		self
	}

	/// Sets the 'clean session' flag to send to the broker.
	///
	/// # Arguments
	///
	/// `clean` Whether the broker should remove any previously-stored 
	///			information for this client.
	pub fn clean_session(&mut self, clean: bool) -> &mut ConnectOptionsBuilder {
		self.copts.cleansession = if clean { 1 } else { 0 };
		self
	}

	/// Sets the maximum number of in-flight messages that can be 
	/// simultaneously handled by this client.
	///
	/// # Arguments
	///
	/// `max_inflight` The maximum number of messages that can be in-flight
	///				   at any given time with this client. 
	pub fn max_inflight(&mut self, max_inflight: i32) -> &mut ConnectOptionsBuilder {
		self.copts.maxInflight = max_inflight;
		self
	}

	/// Sets the LWT options for the connection.
	///
	/// # Arguments
	///
	/// `will` The LWT options for the connection.
	pub fn will_options(&mut self, will: WillOptions) -> &mut ConnectOptionsBuilder {
		self.will = Some(will);
		self
	}

	/// Sets the SSL options for the connection.
	///
	/// # Arguments
	///
	/// `ssl` The SSL options for the connection.
	pub fn ssl_options(&mut self, ssl: SslOptions) -> &mut ConnectOptionsBuilder {
		self.ssl = Some(ssl);
		self
	}

	/// Sets the user name for authentication with the broker.
	/// This works with the password.
	/// 
	/// # Arguments
	///
	/// `user_name` The user name to send to the broker.
	///
	pub fn user_name(&mut self, user_name: &str) -> &mut ConnectOptionsBuilder {
		self.user_name = user_name.to_string();
		self
	}

	/// Sets the password for authentication with the broker.
	/// This works with the user name.
	/// 
	/// # Arguments
	///
	/// `password` The password to send to the broker.
	///
	pub fn password(&mut self, password: &str) -> &mut ConnectOptionsBuilder {
		self.password = password.to_string();
		self
	}

	/// Sets the time interval to allow the connect to complete.
	///
	/// # Arguments
	///
	/// `timeout` The time interval to allow the connect to 
	/// 		  complete. This has a resolution of seconds.
	///
	pub fn connect_timeout(&mut self, timeout: Duration) -> &mut ConnectOptionsBuilder {
		let secs = timeout.as_secs();
		self.copts.connectTimeout = if secs == 0 { 1 } else { secs as i32 };
		self
	}

	/// Sets the retry interval.
	///
	/// # Arguments
	///
	/// `interval` The retry interval. This has a resolution of seconds.
	pub fn retry_interval(&mut self, interval: Duration) -> &mut ConnectOptionsBuilder {
		let secs = interval.as_secs();
		self.copts.connectTimeout = if secs == 0 { 1 } else { secs as i32 };
		self
	}

	/// Specify the servers to which the client will connect.
	///
	/// # Arguments
	///
	/// `server_uris` The addresses of the brokers to which this client 
	/// 			  should connect.
	//
	pub fn server_uris(&mut self, server_uris: Vec<String>) -> &mut ConnectOptionsBuilder {
		self.server_uris = StringCollection::new(&server_uris);
		self
	}

	/// Sets the version of MQTT to use on the connect.
	///
	/// # Arguments
	///
	/// `ver` The version of MQTT to use when connecting to the broker.
	///		  * (0) try the latest version (3.1.1) and work backwards
	///		  * (3) only try v3.1 
	///		  * (4) only try v3.1.1
	///
	pub fn mqtt_version(&mut self, ver: u32) -> &mut ConnectOptionsBuilder {
		self.copts.MQTTVersion = ver as i32;
		self
	}

	/// Sets the client to automatically reconnect if the connection is lost.
	///
	/// # Arguments
	/// `min_retry_interval` The minimum retry interval. Doubled on each 
	/// 					 failed retry. This has a resolution in seconds.
	/// `max_retry_interval` The maximum retry interval. Doubling stops here
	/// 					 on failed retries. This has a resolution in 
	///						 seconds.
	pub fn automatic_reconnect(&mut self, min_retry_interval: Duration,
										  max_retry_interval: Duration)
				-> &mut ConnectOptionsBuilder 
	{
		self.copts.automaticReconnect = 1;	// true

		let mut secs = min_retry_interval.as_secs();
		self.copts.minRetryInterval = if secs == 0 { 1 } else { secs as i32 };

		secs = max_retry_interval.as_secs();
		self.copts.maxRetryInterval = if secs == 0 { 1 } else { secs as i32 };
		self
	}

	/// Finalize the builder to create the connect options.
	pub fn finalize(&self) -> ConnectOptions {
		let opts = ConnectOptions {
			copts: self.copts.clone(),
			will: if let Some(ref will_opts) = self.will {
					println!("Transferring will");
					Some(Box::new(will_opts.clone()))
				}
				else { None },
			ssl: if let Some(ref ssl_opts) = self.ssl {
					println!("Transferring SSL");
					Some(Box::new(ssl_opts.clone()))
				}
				else { None },
			user_name: CString::new(self.user_name.clone()).unwrap(),
			password: CString::new(self.password.clone()).unwrap(),
			server_uris: self.server_uris.clone(),
		};
		ConnectOptions::fixup(opts)
	}
}

/////////////////////////////////////////////////////////////////////////////
//								Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
	use super::*;
	use std::ffi::{CStr};
	use ssl_options::SslOptionsBuilder;

	#[test]
	fn test_new() {
		let opts = ConnectOptions::new();

		assert_eq!([ 'M' as i8, 'Q' as i8, 'T' as i8, 'C' as i8 ], opts.copts.struct_id);
		assert_eq!(5, opts.copts.struct_version);
		assert_eq!(ptr::null(), opts.copts.will);
		// TODO: Should username and password be NULL or empty string
		//assert_eq!(ptr::null(), opts.copts.username);
		//assert_eq!(ptr::null(), opts.copts.password);
		assert_eq!(ptr::null(), opts.copts.ssl);

		assert_eq!(ptr::null_mut(), opts.copts.context);

		assert_eq!(0, opts.copts.serverURIcount);
		assert_eq!(ptr::null(), opts.copts.serverURIs);

		assert_eq!(0, opts.copts.MQTTVersion);
	}

	#[test]
	fn test_ssl() {
		const TRUST_STORE: &str = "some_file.crt";
		let ssl_opts = SslOptionsBuilder::new()
			.trust_store(TRUST_STORE)
			.finalize();

		let opts = ConnectOptionsBuilder::new()
			.ssl_options(ssl_opts)
			.finalize();

		assert!(!opts.copts.ssl.is_null());
			
		if let Some(ref ssl_opts) = opts.ssl {
			// TODO: Test that ssl_opts.get_trust_store() is TRUST_STORE?
			assert!(true);
			assert_eq!(&ssl_opts.copts as *const _, opts.copts.ssl);
			let ts = unsafe { CStr::from_ptr((*opts.copts.ssl).trustStore) };
			assert_eq!(TRUST_STORE, ts.to_str().unwrap());
		}
		else {
			// The SSL option should be set
			assert!(false);
		};
	}

	#[test]
	fn test_user_name() {
		const NAME: &'static str = "some-random-name";

		let opts = ConnectOptionsBuilder::new()
						.user_name(NAME).finalize();

		let s = unsafe { CStr::from_ptr(opts.copts.username) };
		assert_eq!(NAME, s.to_str().unwrap());
	}

	#[test] 
	fn test_server_uris() {
		let servers = vec!("tcp://server1:1883".to_string(), "ssl://server2:1885".to_string());

		let opts = ConnectOptionsBuilder::new()
						.server_uris(servers.clone()).finalize();

		assert_eq!(servers.len() as i32, opts.copts.serverURIcount);

		// Compare the strings to the C-arrays in copts
		for (i, ref svr) in servers.iter().enumerate() {
			let s = unsafe { CStr::from_ptr(*opts.copts.serverURIs.offset(i as isize)) };
			assert_eq!(&svr[..], s.to_str().unwrap());
		}
	}

	#[test]
	fn test_mqtt_version() {
		const VER: u32 = MQTT_VERSION_3_1_1;

		let opts = ConnectOptionsBuilder::new().mqtt_version(VER).finalize();
		assert_eq!(VER as i32, opts.copts.MQTTVersion);
	}

}

