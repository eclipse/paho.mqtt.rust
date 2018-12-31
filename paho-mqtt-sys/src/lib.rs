// lib.rs
//
// The main library file for the Paho MQTT Rust library low-level C wrapper.
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

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Temporary
#![allow(dead_code)]

use std::ptr;
use std::os::raw::{c_char, c_int};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// The following 'Default' trait implementations contain initializations
// for the structures from the Paho C library. Each of those structs
// contains an initializer macro in MQTTAsync.h.
// By convention, these default initializers match those macros from the
// C library.

/////////////////////////////////////////////////////////////////////////////
// Client creation

impl Default for MQTTAsync_createOptions {
	fn default() -> MQTTAsync_createOptions {
		MQTTAsync_createOptions {
			struct_id: [ b'M' as c_char, b'Q' as c_char, b'C' as c_char, b'O' as c_char],
			struct_version: 0,
			sendWhileDisconnected: 0,
			maxBufferedMessages: 100,
            MQTTVersion: MQTTVERSION_DEFAULT as c_int,
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
// Connecting

impl Default for MQTTAsync_connectOptions {
	fn default() -> MQTTAsync_connectOptions {
		MQTTAsync_connectOptions {
			struct_id: [ b'M' as c_char, b'Q' as c_char, b'T' as c_char, b'C' as c_char],
			struct_version: 5,
			keepAliveInterval: 60,
			cleansession: 1,
			maxInflight: 10,
			will: ptr::null_mut(),
			username: ptr::null(),
			password: ptr::null(),
			connectTimeout: 30,
			retryInterval: 0,
			ssl: ptr::null_mut(),
			onSuccess: None,
			onFailure: None,
			context: ptr::null_mut(),
			serverURIcount: 0,
			serverURIs: ptr::null(),
			MQTTVersion: 0,
			automaticReconnect: 0,
			minRetryInterval: 1,
			maxRetryInterval: 60,
			binarypwd: MQTTAsync_connectOptions__bindgen_ty_1 {
				len: 0,
				data: ptr::null(),
			},
            cleanstart: 0,
            connectProperties: ptr::null_mut(),
            willProperties: ptr::null_mut(),
            onSuccess5: None,
            onFailure5: None,
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
// Options

impl Default for MQTTAsync_willOptions {
	fn default() -> MQTTAsync_willOptions {
		MQTTAsync_willOptions {
			struct_id: [ b'M' as c_char, b'Q' as c_char, b'T' as c_char, b'W' as c_char ],
			struct_version: 1,	// 1 indicates binary payload
			topicName: ptr::null(),
			message: ptr::null(),
			retained: 0,
			qos: 0,
			payload: MQTTAsync_willOptions__bindgen_ty_1 {
				len: 0,
				data: ptr::null(),
			}
		}
	}
}

impl Default for MQTTAsync_SSLOptions {
	fn default() -> MQTTAsync_SSLOptions {
		MQTTAsync_SSLOptions {
			struct_id: [ b'M' as c_char, b'Q' as c_char, b'T' as c_char, b'S' as c_char ],
			struct_version: 0,
			trustStore: ptr::null(),
			keyStore: ptr::null(),
			privateKey: ptr::null(),
			privateKeyPassword: ptr::null(),
			enabledCipherSuites: ptr::null(),
			enableServerCertAuth: 1,
			sslVersion: 1,
			verify: 0,
			CApath: ptr::null(),
            ssl_error_cb: None,
            ssl_error_context: ptr::null_mut(),
		}
	}
}

// New for MQTT v5
impl Default for MQTTSubscribe_options {
	fn default() -> MQTTSubscribe_options {
		MQTTSubscribe_options {
			struct_id: [ b'M' as c_char, b'Q' as c_char, b'S' as c_char, b'O' as c_char ],
            struct_version: 0,
            noLocal: 0,
            retainAsPublished: 0,
            retainHandling: 0,
        }
    }
}


impl Default for MQTTAsync_responseOptions {
	fn default() -> MQTTAsync_responseOptions {
		MQTTAsync_responseOptions {
			struct_id: [ b'M' as c_char, b'Q' as c_char, b'T' as c_char, b'R' as c_char ],
			struct_version: 0,
			onSuccess: None,
			onFailure: None,
			context: ptr::null_mut(),
			token: 0,
            onSuccess5: None,
            onFailure5: None,
            properties: MQTTProperties::default(),
            subscribeOptions: MQTTSubscribe_options::default(),
            subscribeOptionsCount: 0,
            subscribeOptionsList: ptr::null_mut(),
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
// MQTTProperties (new for v5)

impl Default for MQTTProperties {
    fn default() -> MQTTProperties {
        MQTTProperties {
            count: 0,
            max_count: 0,
            length: 0,
            array: ptr::null_mut(),
        }
    }
}



/////////////////////////////////////////////////////////////////////////////
// Messages

impl Default for MQTTAsync_message {
	fn default() -> MQTTAsync_message {
		MQTTAsync_message {
			struct_id: [ b'M' as c_char, b'Q' as c_char, b'T' as c_char, b'M' as c_char ],
			struct_version: 0,
			payloadlen: 0,
			payload: ptr::null_mut(),
			qos: 0,
			retained: 0,
			dup: 0,
			msgid: 0,
            properties: MQTTProperties::default(),
		}
	}	
}

/////////////////////////////////////////////////////////////////////////////
// Disconnecting

impl Default for MQTTAsync_disconnectOptions {
	fn default() -> MQTTAsync_disconnectOptions {
		MQTTAsync_disconnectOptions {
			struct_id: [ b'M' as c_char, b'Q' as c_char, b'T' as c_char, b'D' as c_char],
			struct_version: 0,
			timeout: 0,
			onSuccess: None,
			onFailure: None,
			context: ptr::null_mut(),
            properties: MQTTProperties::default(),
            reasonCode: 0,
            onSuccess5: None,
            onFailure5: None,
		}
	}
}

/////////////////////////////////////////////////////////////////////////////

impl Default for MQTTClient_persistence {
    fn default() -> Self {
        MQTTClient_persistence {
            context: ptr::null_mut(),
            popen: None,
            pclose: None,
            pput: None,
            pget: None,
            premove: None,
            pkeys: None,
            pclear: None,
            pcontainskey: None,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
// Unit Tests

#[cfg(test)]
mod tests {
}

