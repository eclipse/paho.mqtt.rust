// response_options.rs
//
// The set of options for responses coming back to an MQTT client.
// This file is part of the Eclipse Paho MQTT Rust Client library.
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

//! Response options for the Paho MQTT Rust client library.

use ffi;

use token::Token;

/// The collection of options for responses coming back to the client.
#[derive(Debug)]
pub struct ResponseOptions {
    pub(crate) copts: ffi::MQTTAsync_responseOptions,
}

impl ResponseOptions {
    pub fn new(tok: &Token) -> Self {
        let rsp_tok = tok.clone();
        ResponseOptions {
            copts: ffi::MQTTAsync_responseOptions {
                onSuccess: Some(Token::on_success),
                onFailure: Some(Token::on_failure),
                context: Token::into_raw(rsp_tok),
                ..ffi::MQTTAsync_responseOptions::default()
            }
        }
    }
}


