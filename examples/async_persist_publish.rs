// paho-mqtt/examples/async_persist_publish.rs
// Example application for Paho MQTT Rust library.
//
//! This is an asynchronous MQTT publisher with user-defined, in-memory,
//! persistence using the Paho Rust client library.
//!
//! This example demonstrates:
//!   - Creating a client using CreateOptionsBuilder
//!   - User-defined persistence, in-memory, with a Rust HashMap
//!   - Connecting to an MQTT broker
//!   - Publishing messages asynchronously
//!   - Using the Rust logger to trace the persistence callbacks.
//!

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

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate paho_mqtt as mqtt;

use std::process;
use std::collections::HashMap;

// Use a non-zero QOS to exercise the persistence store
const QOS: i32 = 1;

// --------------------------------------------------------------------------

/// The ClientPersistence maps pretty closely to a key/val store. We can use
/// a Rust HashMap to implement an in-memory persistence pretty easily.
/// The keys are strings, and the values are arbitrarily-sized byte buffers.
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
    // Open the persistence store.
    // We don't need to do anything here since the store is in memory.
    // We just capture the name for logging/debugging purposes.
    fn open(&mut self, client_id: &str, server_uri: &str) -> mqtt::MqttResult<()> {
        self.name = format!("{}-{}", client_id, server_uri);
        trace!("Client persistence [{}]: open", self.name);
        Ok(())
    }

    // Close the persistence store.
    // We don't need to do anything.
    fn close(&mut self) -> mqtt::MqttResult<()> {
        trace!("Client persistence [{}]: close", self.name);
        Ok(())
    }

    // Put data into the persistence store.
    // We get a vector of buffer references for the data to store, which we
    // can concatenate into a single byte buffer to place in the map.
    fn put(&mut self, key: &str, buffers: Vec<&[u8]>) -> mqtt::MqttResult<()> {
        trace!("Client persistence [{}]: put key '{}'", self.name, key);
        let buf: Vec<u8> = buffers.concat();
        self.map.insert(key.to_string(), buf);
        Ok(())
    }

    // Get (retrieve) data from the persistence store.
    // We look up and return any data corresponding to the specified key.
    fn get(&self, key: &str) -> mqtt::MqttResult<Vec<u8>> {
        trace!("Client persistence [{}]: get key '{}'", self.name, key);
        match self.map.get(key) {
            Some(v) => Ok(v.to_vec()),
            None => Err(mqtt::PERSISTENCE_ERROR)
        }
    }

    // Remove the key entry from the persistence store, if any.
    fn remove(&mut self, key: &str) -> mqtt::MqttResult<()> {
        trace!("Client persistence [{}]: remove key '{}'", self.name, key);
        match self.map.remove(key) {
            Some(_) => Ok(()),
            None => Err(mqtt::PERSISTENCE_ERROR)
        }
    }

    // Retrieve the complete set of keys in the persistence store.
    fn keys(&self) -> mqtt::MqttResult<Vec<String>> {
        trace!("Client persistence [{}]: keys", self.name);
        let mut keys: Vec<String> = Vec::new();
        for key in self.map.keys() {
            keys.push(key.to_string());
        }
        debug!("Found keys: {:?}", keys);
        Ok(keys)
    }

    // Clears all the data from the persistence store.
    fn clear(&mut self) -> mqtt::MqttResult<()> {
        trace!("Client persistence [{}]: clear", self.name);
        self.map.clear();
        Ok(())
    }

    // Determine if the persistence store contains the specified key.
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

    let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
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
    let msg = mqtt::Message::new("test", "Hello world!", QOS);
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
