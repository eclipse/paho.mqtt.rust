// ssl_publish.rs
// 
// This is a Paho MQTT Rust client, sample application.
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

// We can test this using mosquitto configured with certificates in the
// Paho C library. The C library has an SSL/TSL test suite, and we can use
// that to test:
//     mosquitto -c paho.mqtt.c/test/tls-testing/mosquitto.conf
//
// Then use the file "test-root-ca.crt" from that directory
// (paho.mqtt.c/test/tls-testing) for the trust store for this program.
//

extern crate paho_mqtt;
extern crate paho_mqtt3as_sys as ffi;

use std::{time, env, process};
use std::path::Path;
use std::ffi::CStr;
use paho_mqtt as mqtt;

fn main() {
	const TRUST_STORE: &str = "/home/fmp/mqtt/paho-rust/test-root-ca.crt";

	// We assume that we are in a valid directory.
	let cwd = env::current_dir().unwrap();
	println!("The current directory is {}", cwd.display());

	let trust_store = Path::new(TRUST_STORE);
	if !trust_store.exists() {
		println!("The trust store file does not exist: {}", TRUST_STORE);
		process::exit(1);
	}
	
	// Create a client & define connect options
	let mut cli = mqtt::AsyncClientBuilder::new()
			.server_uri("ssl://localhost:18885")
			.client_id("ssl_publish_rs")
			.offline_buffering(true)
			.finalize();

	let ssl_opts = mqtt::SslOptionsBuilder::new()
		.trust_store(TRUST_STORE)
		.finalize();

	println!("SSL: {:?}", ssl_opts);
	let mut ts = unsafe { CStr::from_ptr(ssl_opts.copts.trustStore) };
	println!("Main SSL Opts Trust Store: {:?}", ts);

	let conn_opts = mqtt::ConnectOptionsBuilder::new()
		.ssl_options(ssl_opts)
		.user_name("testuser")
		.password("testpassword")
		.finalize();

	println!("Connect options: {:?}", conn_opts);

	ts = unsafe { CStr::from_ptr((*conn_opts.copts.ssl).trustStore) };
	unsafe {
		println!("Main Conn Trust Store: [{:?}] {:?}", (*conn_opts.copts.ssl).trustStore, ts);
	}

	let tok = cli.connect(conn_opts);
	if let Err(e) = tok.wait() {
		println!("Error connecting: {:?}", e);
		::std::process::exit(1);
	}

	println!("");

	let msg = mqtt::MessageBuilder::new()
		.topic("test")
		.payload("Hello secure world!".as_bytes())
		.qos(1)
		.finalize();

	if let Err(e) = cli.publish(msg).wait() {
		println!("Error sending message: {:?}", e);
	}

	let disconn_opts = mqtt::DisconnectOptionsBuilder::new()
		.timeout(time::Duration::from_millis(1000))
		.finalize();

	let tok = cli.disconnect(disconn_opts);
	tok.wait().unwrap();
}

