// lib.rs
//
// The main library file for the Paho MQTT Rust Library.
//

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Temporary
#![allow(dead_code)]

use std::ptr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/////////////////////////////////////////////////////////////////////////////
// Client creation

impl Default for MQTTAsync_createOptions {
	fn default() -> MQTTAsync_createOptions {
		MQTTAsync_createOptions {
			struct_id: [ 'M' as i8, 'Q' as i8, 'C' as i8, 'O' as i8],
			struct_version: 0,
			sendWhileDisconnected: 0,
			maxBufferedMessages: 100,
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
// Connecting

impl Default for MQTTAsync_connectOptions {
	fn default() -> MQTTAsync_connectOptions {
		MQTTAsync_connectOptions {
			struct_id: [ 'M' as i8, 'Q' as i8, 'T' as i8, 'C' as i8],
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
			}
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
// Options

// Defaults for MQTTAsync_willOptions
// These default options should match the C macro: 
//   MQTTAsync_willOptions_initializer
impl Default for MQTTAsync_willOptions {
	fn default() -> MQTTAsync_willOptions {
		MQTTAsync_willOptions {
			struct_id: [ 'M' as i8, 'Q' as i8, 'T' as i8, 'W' as i8 ],
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

// Defaults for MQTTAsync_SSLOptions
// These default options should match the C language macro:
//		MQTTAsync_SSLOptions_initializer
impl Default for MQTTAsync_SSLOptions {
	fn default() -> MQTTAsync_SSLOptions {
		MQTTAsync_SSLOptions {
			struct_id: [ 'M' as i8, 'Q' as i8, 'T' as i8, 'S' as i8 ],
			struct_version: 0,
			trustStore: ptr::null(),
			keyStore: ptr::null(),
			privateKey: ptr::null(),
			privateKeyPassword: ptr::null(),
			enabledCipherSuites: ptr::null(),
			enableServerCertAuth: 1,
		}
	}
}


impl Default for MQTTAsync_responseOptions {
	fn default() -> MQTTAsync_responseOptions {
		MQTTAsync_responseOptions {
			struct_id: [ 'M' as i8, 'Q' as i8, 'T' as i8, 'R' as i8 ],
			struct_version: 0,
			onSuccess: None,
			onFailure: None,
			context: ptr::null_mut(),
			token: 0,
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
// Messages

impl Default for MQTTAsync_message {
	fn default() -> MQTTAsync_message {
		MQTTAsync_message {
			struct_id: [ 'M' as i8, 'Q' as i8, 'T' as i8, 'M' as i8 ],
			struct_version: 0,
			payloadlen: 0,
			payload: ptr::null_mut(),
			qos: 0,
			retained: 0,
			dup: 0,
			msgid: 0
		}
	}	
}

/////////////////////////////////////////////////////////////////////////////
// Disconnecting

impl Default for MQTTAsync_disconnectOptions {
	fn default() -> MQTTAsync_disconnectOptions {
		MQTTAsync_disconnectOptions {
			struct_id: [ 'M' as i8, 'Q' as i8, 'T' as i8, 'D' as i8],
			struct_version: 0,
			timeout: 0,
			onSuccess: None,
			onFailure: None,
			context: ptr::null_mut(),
		}
	}
}

/////////////////////////////////////////////////////////////////////////////
// Unit Tests

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

