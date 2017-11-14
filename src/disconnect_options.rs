// diconnect_options.rs
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
use std::time::Duration;

/// The collection of options for disconnecting from the client.
#[derive(Debug)]
pub struct DisconnectOptions {
	pub copts: ffi::MQTTAsync_disconnectOptions,
}

impl DisconnectOptions {
	pub fn new() -> DisconnectOptions {
		DisconnectOptions::default()
	}
}

impl Default for DisconnectOptions {
	fn default() -> DisconnectOptions {
		DisconnectOptions {
			copts: ffi::MQTTAsync_disconnectOptions::default(),
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
//								Builder
/////////////////////////////////////////////////////////////////////////////

pub struct DisconnectOptionsBuilder {
	copts: ffi::MQTTAsync_disconnectOptions,
}

impl DisconnectOptionsBuilder {
	pub fn new() -> DisconnectOptionsBuilder {
		DisconnectOptionsBuilder {
			copts: ffi::MQTTAsync_disconnectOptions::default(),
		}
	}

	/// Sets the time interval to allow the disconnect to complete.
	/// This specifies the time to allow in-flight messages to complete.
	/// 
	/// # Arguments
	///
	/// `timeout` The time interval to allow the disconnect to 
	/// 		  complete. This has a resolution of seconds.
	pub fn timeout(&mut self, timeout: Duration) -> &mut DisconnectOptionsBuilder {
		let secs = timeout.as_secs();
		self.copts.timeout = if secs == 0 { 1 } else { secs as i32 };
		self
	}

	/// Finalize the builder to create the connect options.
	pub fn finalize(&self) -> DisconnectOptions {
		DisconnectOptions {
			copts: self.copts.clone(),
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
//								Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
	use super::*;
	//use std::ffi::{CStr};

	#[test]
	fn test_new() {
		let opts = DisconnectOptions::new();

		assert_eq!([ 'M' as i8, 'Q' as i8, 'T' as i8, 'D' as i8 ], opts.copts.struct_id);
		assert_eq!(0, opts.copts.struct_version);
	}

}

