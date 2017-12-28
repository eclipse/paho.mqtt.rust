// topic.rs
// 
// A set of message parameters to repeatedly publish to the same topic.
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
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


use std::sync::{Arc};
//use ffi;
use async_client::{AsyncClient,DeliveryToken};
use message::Message;
//use errors::{MqttResult};

/////////////////////////////////////////////////////////////////////////////
//								Topic
/////////////////////////////////////////////////////////////////////////////

pub struct Topic<'a> {
	cli: &'a AsyncClient,
	topic: String,
	qos: i32,
	retained: bool,
}

impl<'a> Topic<'a> 
{
	pub fn new(cli: &'a AsyncClient, topic: &str, qos: i32) -> Topic<'a> {
		Topic {
			cli,
			topic: topic.to_string(),
			qos,
			retained: false,
		}
	}

	pub fn publish<V>(&self, payload: V) -> Arc<DeliveryToken>
		where V: Into<Vec<u8>>
	{
		let msg = Message::new(&self.topic, payload, self.qos);
		self.cli.publish(msg)
	}
}


