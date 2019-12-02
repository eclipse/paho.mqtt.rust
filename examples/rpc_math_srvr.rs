// paho-mqtt/examples/rpc_math_srvr.rs
//
//! This is a Paho MQTT v5 C++ sample application.
//!
//! It's an example of how to create an RPC server client for servicing
//! remote procedure calls using MQTT with the 'response topic' and 'correlation data'
//! properties.
//!
//! The sample demonstrates:
//!  - Creating an RPC service for MQTT v5
//!  - Connecting to an MQTT v5 server/broker
//!  - Using MQTT v5 properties
//!  - Receiving RPC request messages, and sending replies.
//!  - Using asynchronous tokens
//!	 - Subscribing to request topic
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
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate paho_mqtt as mqtt;

//use std::env;
use std::process;
use std::collections::HashMap;
use futures::Future;
use serde_json::json;


const QOS: i32 = 1;
const MQTTV5: u32 = 5;

// The RPC function implementations

fn add(args: &[f64]) -> f64 { args.iter().sum() }
fn mult(args: &[f64]) -> f64 { args.iter().product() }

/// The math function signature.
type MathFn = fn(args: &[f64]) -> f64;

// A table of names to functions.
lazy_static! {
    static ref FUNC_TBL: HashMap<&'static str, MathFn> = {
        let mut tbl = HashMap::new();
        tbl.insert("add", add as MathFn);
        tbl.insert("mult", mult as MathFn);
        tbl
    };
}

// --------------------------------------------------------------------------

// The request topics will be of the form:
//     "requests/math/<operation>"
// where <operation> ("add", "mult", etc) tells us which processing function
// to use.

fn handle_request(cli: &mqtt::AsyncClient, msg: mqtt::Message) -> mqtt::MqttResult<()>
{
    //println!("Request Message: {:?}", msg);

    let reply_to = msg.properties()
        .get_string(mqtt::PropertyCode::RESPONSE_TOPIC)
    .ok_or(mqtt::MqttError::from("No response topic provided."))?;

    let corr_id = msg.properties()
        .get_binary(mqtt::PropertyCode::CORRELATION_DATA)
        .ok_or(mqtt::MqttError::from("No correlation data provided."))?;

    println!("\nRequest w/ Reply To: {}, Correlation ID: {:?}", reply_to, corr_id);

    // Get the name of the function to call from the topic

    let topic_arr: Vec<&str> = msg.topic().split("/").collect();

    if topic_arr.len() < 3 {
        return Err("Malformed request topic".into());
    }

    let fname = topic_arr[2];

    // Get the parameters from the payload

    let params: Vec<f64> = serde_json::from_str(&msg.payload_str()).unwrap();

    // Look up the function, by name, and call if found

    if let Some(func) = FUNC_TBL.get(fname) {
        println!("{}: {:?}", fname, params);
        let x = func(&params);
        println!("    Result: {}", x);

        // Form a reply message, using the correlation ID

        let mut props = mqtt::Properties::new();
        props.push_binary(mqtt::PropertyCode::CORRELATION_DATA, corr_id).unwrap();

        let payload = json!(x).to_string();

        // Create a message and publish it
        let msg = mqtt::MessageBuilder::new()
            .topic(reply_to)
            .payload(payload)
            .qos(QOS)
            .properties(props)
            .finalize();

        let _ = cli.publish(msg);
    }
    else {
        eprintln!("Unknown command: {}", fname);
    }
    Ok(())
}

// --------------------------------------------------------------------------

fn main() -> mqtt::MqttResult<()> {
    // Initialize the logger from the environment
    env_logger::init();

    // We use the broker on this host.
    let host = "localhost";

    // Command-line option(s)
    //let args: Vec<String> = env::args().skip(1).collect();

    const REQ_TOPIC_HDR: &'static str = "requests/math/#";

    // Create a client to the specified host, no persistence
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .mqtt_version(MQTTV5)
        .server_uri(host)
        .client_id("rpc_math_srvr")
        .persistence(mqtt::PersistenceType::None)
        .finalize();


    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
        eprintln!("Error creating the client: {}", err);
        process::exit(1);
    });

    // Initialize the consumer before connecting.
    // With a clean session/start, this order isn't important,
    // but it's still a good habit to start consuming first.
    let rx = cli.start_consuming();

    // Connect with default options
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .mqtt_version(MQTTV5)
        .clean_start(false)
        .finalize();

    // Connect and wait for it to complete or fail

    let rsp = cli.connect(conn_opts).wait().unwrap_or_else(|err| {
        eprintln!("Unable to connect: {:?}", err);
        process::exit(1);
    });

    // We're connecting with a persistent session. So we check if
    // the server already knows about us and rembers about out
    // subscription(s). If not, we subscribe for incoming requests.

    if let Some((_uri, _ver, session)) = rsp.connect_response() {
        if !session {
            println!("Subscribing to math requests");
            cli.subscribe(REQ_TOPIC_HDR, QOS).wait()?;
        }
    }


    for msg in rx.iter() {
        if let Some(msg) = msg {
            if let Err(err) = handle_request(&cli, msg) {
                eprintln!("Error: {}", err);
            }
        }
        else {
            eprintln!("Error receiving reply.");
        }
    }

    // Disconnect from the broker
    cli.disconnect(None).wait()?;
    Ok(())
}

