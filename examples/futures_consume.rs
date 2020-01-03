// paho-mqtt/examples/futures_consume.rs
// This is a Paho MQTT Rust client, sample application.
//
//! This application is an MQTT subscriber using the asynchronous futures
//! stream interface of the Paho Rust client library, employing callbacks
//! to receive messages and status updates. It also monitors for disconnects
//! and performs manual re-connections.
//!
//! The sample demonstrates:
//!   - Futures stream for receiving messages
//!   - Connecting to an MQTT server/broker.
//!   - Checking server responses
//!   - Subscribing to multiple topics
//!   - Using automatic reconnect to let the underlying library reconnect to
//!     the server on lost connections
//!   - Using a persistent (non-clean) session to get all messages even with
//!     transient lost connections.
//!   - Last will and testament
//!

/*******************************************************************************
 * Copyright (c) 2018 Frank Pagliughi <fpagliughi@mindspring.com>
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
extern crate futures;
extern crate paho_mqtt as mqtt;

use std::{env, process};
use std::time::Duration;

use futures::{Future, Stream};
use futures::future::{ok, err};

// The topics to which we subscribe.
const TOPICS: &[&str] = &[ "test", "hello" ];
const QOS: &[i32] = &[1, 1];

/////////////////////////////////////////////////////////////////////////////

fn main() {
    // Initialize the logger from the environment
    env_logger::init();

    // Get a host URL from the command line, if any
    let host = env::args().nth(1).unwrap_or_else(||
        "tcp://localhost:1883".to_string()
    );

    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("rust_futures_consumer")
        .finalize();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    // Define the set of options for the connection (persistent session,
    // auto reconnect, Last Will and Testament (lwt), etc)
    let lwt = mqtt::Message::new("failure", "Futures consumer lost connection", 1);

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .keep_alive_interval(Duration::from_secs(20))
        .automatic_reconnect(Duration::from_millis(500), Duration::from_secs(30))
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    // Open the receiver stream before we even try to connect. This
    // insures that we don't miss any messages, even when a restart of the
    // app gets us a flood of stored messages from the persistent session.
    let rx = cli.get_stream(10);

    // Make the connection to the broker
    println!("Connecting to the MQTT server...");
    cli.connect(conn_opts)
        .and_then(|rsp| {
            let mut fut = mqtt::Token::from_success();
            if let Some((server_uri, ver, session_present)) = rsp.connect_response() {
                println!("Connected to: '{}' with MQTT version {}", server_uri, ver);
                if !session_present {
                    // Subscribe to multiple topics
                    println!("Subscribing to topics: {:?}", TOPICS);
                    fut = cli.subscribe_many(TOPICS, QOS)
                }
            }
            fut
        })
        .and_then(|rsp| {
            if let Some(qosv) = rsp.subscribe_many_response() {
                if !qosv.is_empty() {
                    println!("QoS granted: {:?}", qosv);
                }
            }
            mqtt::Token::from_success()
        })
        .wait().unwrap_or_else(|err| {
            println!("Error: {}", err);
            process::exit(2);
        });

    // Just wait for incoming messages by running the receiver stream
    // in this thread.
    println!("Waiting for messages...");
    rx.for_each(|opt_msg| {
        if let Some(msg) = opt_msg {
            println!("{}", msg);
            ok(())
        }
        else {
            println!("Stream disruption");
            err(())
        }
    }).wait().unwrap_or_else(|_| {
        println!("Done");
    });

    // Hitting ^C will exit the app and cause the broker to publish
    // the LWT message since we're not disconnecting cleanly.
}

