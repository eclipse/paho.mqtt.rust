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

use std::os::raw::c_int;

use ffi;
use token::{Token, TokenInner};
use subscribe_options::SubscribeOptions;

/// The collection of options for responses coming back to the client.
#[derive(Debug)]
pub struct ResponseOptions {
    pub(crate) copts: ffi::MQTTAsync_responseOptions,
    sub_opts: Vec<ffi::MQTTSubscribe_options>,
}

impl ResponseOptions {
    /// Creates a `ResponseOptions` intance for the provided token.
    /// The option's `copts` can be passed to one of the C library's
    /// asynchronous functions to set the callbacks to complete the token.
    ///
    /// Note: We leak an Arc clone of the token into the internal C
    /// structure to act as the context pointer for the callback. It is
    /// up to the callback (or calling function) to recapture and release
    /// this token.
    pub(crate) fn new<T>(tok: T, mqtt_version: u32) -> Self
        where T: Into<Token>
    {
        let tok = tok.into();
        let context = tok.into_raw();
        debug!("Created response for token at: {:?}", context);

        if mqtt_version < 5 {
            ResponseOptions {
                copts: ffi::MQTTAsync_responseOptions {
                    onSuccess: Some(TokenInner::on_success),
                    onFailure: Some(TokenInner::on_failure),
                    context,
                    ..ffi::MQTTAsync_responseOptions::default()
                },
                sub_opts: Vec::new(),
            }
        }
        else {
            ResponseOptions {
                copts: ffi::MQTTAsync_responseOptions {
                    onSuccess5: Some(TokenInner::on_success5),
                    onFailure5: Some(TokenInner::on_failure5),
                    context,
                    ..ffi::MQTTAsync_responseOptions::default()
                },
                sub_opts: Vec::new(),
            }
        }
    }

    pub(crate) fn from_subscribe_options<T>(tok: T, opts: SubscribeOptions) -> Self
        where T: Into<Token>
    {
        let mut ropts = ResponseOptions::new(tok, ffi::MQTTVERSION_5);
        ropts.copts.subscribeOptions = opts.copts;
        ropts
    }

    pub(crate) fn from_subscribe_many_options<T>(tok: T, opts: &[SubscribeOptions]) -> Self
        where T: Into<Token>
    {
        let mut ropts = ResponseOptions::new(tok, ffi::MQTTVERSION_5);
        ropts.sub_opts = opts.iter().map(|opt| opt.copts).collect();
        ropts.copts.subscribeOptionsCount = opts.len() as c_int;
        ropts
    }
}


/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use token::{Token};
    use types::*;

    #[test]
    fn test_new() {
        let tok = Token::new();
        let opts = ResponseOptions::new(tok.clone(), MQTT_VERSION_3_1_1);   //ffi::MQTTVERSION_3_1_1);

        let inner = Token::into_raw(tok);

        assert!(opts.copts.onSuccess.is_some());
        assert!(opts.copts.onFailure.is_some());
        // Check that the context is pointing to the right Token
        assert_eq!(inner, opts.copts.context);

        let _ = unsafe { Token::from_raw(inner) };
    }
}

