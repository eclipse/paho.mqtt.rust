// errors.rs
// This file is part of the Eclipse Paho MQTT Rust Client library.

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

use std::{
    io,
    result,
    str,
};
use thiserror::Error;
use crate::ffi;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{}", error_message(*.0))]
    Paho(i32),
    #[error("[{0}] {1}")]
    PahoDescr(i32, String),
    #[error("Reason code: {0}")]
    ReasonCode(i32),
    #[error("I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("String UTF-8 Error")]
    Utf8(#[from] str::Utf8Error),
    #[error("Timeout")]
    Timeout,
    #[error("{0}")]
    General(&'static str),
    #[error("{0}")]
    GeneralString(String),
}

impl From<i32> for Error {
    fn from(rc: i32) -> Error {
        Error::Paho(rc)
    }
}

impl From<&'static str> for Error {
    fn from(descr: &'static str) -> Error {
        Error::General(descr)
    }
}

impl From<String> for Error {
    fn from(descr: String) -> Error {
        Error::GeneralString(descr)
    }
}



pub type Result<T> = result::Result<T, Error>;

// Gets the string associated with the error code from the C lib.
pub fn error_message(rc: i32) -> &'static str {
    match rc {
        ffi::MQTTASYNC_FAILURE => "General failure",
        ffi::MQTTASYNC_PERSISTENCE_ERROR /* -2 */ => "Persistence error",
        ffi::MQTTASYNC_DISCONNECTED => "Client disconnected",
        ffi::MQTTASYNC_MAX_MESSAGES_INFLIGHT => "Maximum inflight messages",
        ffi::MQTTASYNC_BAD_UTF8_STRING => "Bad UTF8 string",
        ffi::MQTTASYNC_NULL_PARAMETER => "NULL Parameter",
        ffi::MQTTASYNC_TOPICNAME_TRUNCATED => "Topic name truncated",
        ffi::MQTTASYNC_BAD_STRUCTURE => "Bad structure",
        ffi::MQTTASYNC_BAD_QOS => "Bad QoS",
        ffi::MQTTASYNC_NO_MORE_MSGIDS => "No more message ID's",
        ffi::MQTTASYNC_OPERATION_INCOMPLETE => "Operation incomplete",
        ffi::MQTTASYNC_MAX_BUFFERED_MESSAGES => "Max buffered messages",
        ffi::MQTTASYNC_SSL_NOT_SUPPORTED => "SSL not supported by Paho C library",
        ffi::MQTTASYNC_BAD_PROTOCOL => "Bad protocol",
        ffi::MQTTASYNC_BAD_MQTT_OPTION => "Bad option",
        ffi::MQTTASYNC_WRONG_MQTT_VERSION => "Wrong MQTT version",
        ffi::MQTTASYNC_0_LEN_WILL_TOPIC => "Zero length Will Topic",
         _ => "Unknown Error",
    }
}

