// paho-mqtt/examples/rpc_math_cli.rs
//
//! This is a Paho MQTT v5 C++ sample application.
//!
//! It's an example of how to create a client for performing remote procedure
//! calls using MQTT with the 'response topic' and 'correlation data'
//! properties.
//!
//! The sample demonstrates:
//!  - Connecting to an MQTT server/broker
//!  - Using MQTT v5 properties
//!  - Publishing RPC request messages
//!  - Using asynchronous tokens
//!	 - Subscribing to reply topic
//

/*******************************************************************************
 * Copyright (c) 2019 Frank Pagliughi <fpagliughi@mindspring.com>
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

extern crate futures;
extern crate log;
extern crate env_logger;
extern crate paho_mqtt as mqtt;

use std::{env, process};
use futures::Future;

fn main() {
    // Initialize the logger from the environment
    env_logger::init();

    // We use this server.
    let host = "localhost";

    // Command-line option(s)
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        println!("USAGE: rpc_math_cli <add|mult> <num1> <num2> [... numN]");
        process::exit(1);
    }

    const QOS: i32 = 1;
    const REQ_TOPIC_HDR: &'static str = "requests/math";

    // Create a client to the specified host, no persistence
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .mqtt_version(5)
        .server_uri(host)
        .persistence(mqtt::PersistenceType::None)
        .finalize();

    println!("Create opts: {:?}", create_opts);

    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {}", err);
        process::exit(1);
    });

    // Initialize the consumer before connecting.
    let _rx = cli.start_consuming();

    // Connect with default options
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .mqtt_version(5)
        .clean_start(true)
        .finalize();

    //println!("Conn Opts: {:?}", conn_opts);

    // Connect and wait for it to complete or fail
    if let Err(e) = cli.connect(conn_opts).wait() {
        println!("Unable to connect: {:?}", e);
        process::exit(1);
    }

    let req = args[0].to_string();
    let req_topic = format!("{}/{}", REQ_TOPIC_HDR, req);

    let mut props = mqtt::Properties::new();
    props.push(mqtt::Property::new_string(mqtt::PropertyCode::RESPONSE_TOPIC, &req_topic).unwrap());
    props.push(mqtt::Property::new_binary(mqtt::PropertyCode::CORRELATION_DATA, "1").unwrap());

    // Create a message and publish it
    println!("Publishing a message on the 'test' topic");
    //let mut msg = mqtt::Message::new("test", "Hello Rust MQTT world!", QOS);
    //msg.set_properties(props);

    let msg = mqtt::MessageBuilder::new()
        .topic(req_topic)
        .payload("math request")
        .qos(QOS)
        .properties(props)
        .finalize();

    let tok = cli.publish(msg);

    if let Err(e) = tok.wait() {
        println!("Error sending message: {:?}", e);
    }

    // Disconnect from the broker
    let tok = cli.disconnect(None);
    tok.wait().unwrap();
}

