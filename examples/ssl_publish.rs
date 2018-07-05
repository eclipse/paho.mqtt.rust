// paho-mqtt/examples/ssl_publish.rs
// Example application for Paho MQTT Rust library.
//
//! This is a simple asynchronous MQTT publisher using SSL/TSL secured
//! connection via the Paho MQTT Rust Library.
//!
//! The sample demonstrates:
//!   - Connecting to an MQTT server/broker securely
//!   - Setting SSL/TLS options
//!   - Publishing messages asynchronously
//!   - Using asynchronous tokens
//!
//! We can test this using mosquitto configured with certificates in the
//! Paho C library. The C library has an SSL/TSL test suite, and we can use
//! that to test:
//!     $ cd paho.mqtt.c
//!     $ mosquitto -c test/tls-testing/mosquitto.conf
//!
//! Then use the file "test-root-ca.crt" from that directory
//! (paho.mqtt.c/test/tls-testing/keys) for the trust store for this program.
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

extern crate log;
extern crate env_logger;
extern crate paho_mqtt as mqtt;

use std::{time, env, process};

fn main() {
    // Initialize the logger from the environment
    env_logger::init().unwrap();

    // We use the trust store from the Paho C tls-testing/keys directory,
    // but we assume there's a copy in the current directory.
    const TRUST_STORE: &str = "test-root-ca.crt";

    // We assume that we are in a valid directory.
    let mut trust_store = env::current_dir().unwrap();
    trust_store.push(TRUST_STORE);

    if !trust_store.exists() {
        println!("The trust store file does not exist: {:?}", trust_store);
        println!("  Get a copy from \"paho.mqtt.c/test/tls-testing/keys/test-root-ca.crt\"");
        process::exit(1);
    }

    // Create a client & define connect options
    let cli = mqtt::AsyncClientBuilder::new()
                    .server_uri("ssl://localhost:18885")
                    .client_id("ssl_publish_rs")
                    .offline_buffering(true)
                    .finalize();

    let ssl_opts = mqtt::SslOptionsBuilder::new()
        .trust_store(trust_store.to_str().unwrap())
        .finalize();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .ssl_options(ssl_opts)
        .user_name("testuser")
        .password("testpassword")
        .finalize();

    let tok = cli.connect(conn_opts);
    if let Err(e) = tok.wait() {
        println!("Error connecting: {:?}", e);
        process::exit(1);
    }

    let msg = mqtt::MessageBuilder::new()
        .topic("test")
        .payload("Hello secure world!")
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

