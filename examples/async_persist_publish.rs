// async_publish.rs
// 
// Example/test for Paho MQTT Rust library.
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

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate paho_mqtt as mqtt;

use std::process;
use std::collections::HashMap;

// --------------------------------------------------------------------------

// The ClientPersistence maps pretty closely to a key/val store. We can use
// a Rust HashMap to implement an in-memory persistence pretty easily.

struct MemPersistence {
	name: String,
	map: HashMap<String, Vec<u8>>,
}

impl MemPersistence {
	fn new() -> MemPersistence {
		MemPersistence {
			name: "".to_string(),
			map: HashMap::new(),
		}
	}
}

impl mqtt::ClientPersistence for MemPersistence
{
	fn open(&mut self, client_id: &str, server_uri: &str) -> mqtt::MqttResult<()> {
		self.name = format!("{}-{}", client_id, server_uri);
		trace!("Client persistence [{}]: open", self.name);

		//self.map.insert("bubba".to_string(), vec![ 0u8, 1u8 ]);
		//self.map.insert("wally".to_string(), vec![ 2u8, 3u8 ]);

		Ok(())
	}

	fn close(&mut self) -> mqtt::MqttResult<()> {
		trace!("Client persistence [{}]: close", self.name);
		Ok(())
	}

	// We get a vector of buffer references for the data to store, which we 
	// can concatenate into a single byte buffer to place in the map.
	fn put(&mut self, key: &str, buffers: Vec<&[u8]>) -> mqtt::MqttResult<()> {
		trace!("Client persistence [{}]: put key '{}'", self.name, key);
		let buf: Vec<u8> = buffers.concat();
		self.map.insert(key.to_string(), buf);
		Ok(())
	}

	fn get(&self, key: &str) -> mqtt::MqttResult<&[u8]> {
		trace!("Client persistence [{}]: get key '{}'", self.name, key);
		match self.map.get(key) {
			Some(v) => Ok(&v),
			None => Err(mqtt::PERSISTENCE_ERROR)
		}
	}

	fn remove(&mut self, key: &str) -> mqtt::MqttResult<()> {
		trace!("Client persistence [{}]: remove key '{}'", self.name, key);
		match self.map.remove(key) {
			Some(_) => Ok(()),
			None => Err(mqtt::PERSISTENCE_ERROR)
		}
	}

	fn keys(&self) -> mqtt::MqttResult<Vec<&str>> {
		trace!("Client persistence [{}]: keys", self.name);
		let mut kv: Vec<&str> = Vec::new();
		for key in self.map.keys() {
			kv.push(key);
		}
		Ok(kv)
	}

	fn clear(&mut self) -> mqtt::MqttResult<()> {
		trace!("Client persistence [{}]: clear", self.name);
		self.map.clear();
		Ok(())
	}

	fn contains_key(&self, key: &str) -> bool {
		trace!("Client persistence [{}]: contains key '{}'", self.name, key);
		self.map.contains_key(key)
	}
}

// --------------------------------------------------------------------------

fn main() {
	// Initialize the logger from the environment
	env_logger::init().unwrap();

	// Create a client & define connect options
	println!("Creating the MQTT client.");
	let create_opts = mqtt::CreateOptionsBuilder::new()
			.server_uri("tcp://localhost:1883")
			.user_persistence(MemPersistence::new())
			.finalize();

	let cli = mqtt::AsyncClient::with_options(create_opts).unwrap_or_else(|e| {
		println!("Error creating the client: {:?}", e);
		process::exit(1);
	});

	let conn_opts = mqtt::ConnectOptions::new();

	// Connect and wait for it to complete or fail
	println!("Connecting to MQTT broker.");
	if let Err(e) = cli.connect(conn_opts).wait() {
		println!("Unable to connect: {:?}", e);
		process::exit(1);
	}

	// Create a message and publish it
	println!("Publishing a message to 'test' topic");
	let msg = mqtt::Message::new("test", "Hello world!");
	let tok = cli.publish(msg);

	if let Err(e) = tok.wait() {
		println!("Error sending message: {:?}", e);
	}

	// Disconnect from the broker
	println!("Disconnecting from the broker.");

	let tok = cli.disconnect(None);
	tok.wait().unwrap();

	println!("Done");
}
