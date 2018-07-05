// diconnect_options.rs
//
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

#![deny(missing_docs)]

//! Disconnect options for the Paho MQTT Rust client library.
//! This contains the structures to define the options for disconnecting from
//! the MQTT broker/server.

use ffi;
use std::time::Duration;

/// The collection of options for disconnecting from the client.
#[derive(Debug)]
pub struct DisconnectOptions {
    /// The underlying C disconnect options
    pub copts: ffi::MQTTAsync_disconnectOptions,
}

impl DisconnectOptions {
    /// Create a new `DisconnectOptions`
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
//                              Builder
/////////////////////////////////////////////////////////////////////////////

/// Builder to create the options for disconnecting from an MQTT server.
pub struct DisconnectOptionsBuilder {
    copts: ffi::MQTTAsync_disconnectOptions,
}

impl DisconnectOptionsBuilder {
    /// Create a new `DisconnectOptionsBuilder`
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
    ///           complete. This has a resolution of seconds.
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
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_char;

    // Identifier fo a C disconnect options struct
    const STRUCT_ID: [c_char; 4] = [ b'M' as c_char, b'Q' as c_char, b'T' as c_char, b'D' as c_char ];

    #[test]
    fn test_new() {
        let opts = DisconnectOptions::new();

        assert_eq!(STRUCT_ID, opts.copts.struct_id);
        assert_eq!(0, opts.copts.struct_version);
    }

}

