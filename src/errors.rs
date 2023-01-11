// errors.rs
//
// Error and Result types for the Paho MQTT Rust library.
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

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

use crate::{ffi, message::Message, reason_code::ReasonCode};
use std::{io, result, str};
use thiserror::Error;

/// The errors from an MQTT operation.
#[derive(Error, Debug)]
pub enum Error {
    /// An error from the underlying Paho C library.
    /// These are brought out as individual constants, below.
    #[error("{}", error_message(*.0))]
    Paho(i32),
    /// An error from the Paho C library with an additional description.
    // TODO: Consider getting rid of this as it makes it more difficult to
    //  match to a Paho Error as sometimes the same error has a description
    //  and other times it doesn't.
    #[error("[{0}] {1}")]
    PahoDescr(i32, String),
    /// A synchronous error when publishing creating or queuing the message.
    #[error("{}", error_message(*.0))]
    Publish(i32, Message),
    /// An MQTT v5 error from a reason code.
    #[error("{0}")]
    ReasonCode(ReasonCode),
    /// A bad topic filter
    #[error("Bad topic filter")]
    BadTopicFilter,
    /// An low-level I/O error
    #[error("I/O failed: {0}")]
    Io(#[from] io::Error),
    /// An error parsing a UTF-8 string
    #[error("String UTF-8 Error")]
    Utf8(#[from] str::Utf8Error),
    /// A string NUL error
    #[error("String NUL Error")]
    Nul(#[from] std::ffi::NulError),
    /// Conversion error between types
    #[error("Conversion Error")]
    Conversion,
    /// A timeout, particularly from a synchronous operation.
    #[error("Timeout")]
    Timeout,
    /// A general error with description
    #[error("{0}")]
    General(&'static str),
    /// A general error with description
    #[error("{0}")]
    GeneralString(String),
}

impl From<i32> for Error {
    /// Create an error from a Paho C return code.
    fn from(rc: i32) -> Error {
        Error::Paho(rc)
    }
}

impl From<&'static str> for Error {
    /// Create a general error from a string.
    fn from(descr: &'static str) -> Error {
        Error::General(descr)
    }
}

impl From<String> for Error {
    /// Create a general error from a string.
    fn from(descr: String) -> Error {
        Error::GeneralString(descr)
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        match err {
            Error::Io(e) => e,
            Error::Timeout => io::Error::new(io::ErrorKind::TimedOut, err),
            _ => io::Error::new(io::ErrorKind::Other, err),
        }
    }
}

/// The result type for MQTT operations.
pub type Result<T> = result::Result<T, Error>;

/// Gets the string associated with the error code from the C lib.
pub fn error_message(rc: i32) -> &'static str {
    match rc {
        ffi::MQTTASYNC_FAILURE => "General failure",
        ffi::MQTTASYNC_PERSISTENCE_ERROR => "Persistence error",
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
        ffi::MQTTASYNC_BAD_MQTT_OPTION => "Bad MQTT option",
        ffi::MQTTASYNC_WRONG_MQTT_VERSION => "Wrong MQTT version",
        ffi::MQTTASYNC_0_LEN_WILL_TOPIC => "Zero length Will Topic",
        ffi::MQTTASYNC_COMMAND_IGNORED => "Command Ignored",
        ffi::MQTTASYNC_MAX_BUFFERED => "Max buffered is zero",
        _ => "Unknown Error",
    }
}

// Some Paho internal error constants

/// A generic error code indicating the failure of an MQTT client operation
pub const Failure: Error = Error::Paho(ffi::MQTTASYNC_FAILURE);

/// An error in the message persistence
pub const PersistenceError: Error = Error::Paho(ffi::MQTTASYNC_PERSISTENCE_ERROR);

/// The client is disconnected
pub const Disconnected: Error = Error::Paho(ffi::MQTTASYNC_DISCONNECTED);

/// The maximum number of messages allowed to be simultaneously in-flight has
/// been reached.
pub const MaxMessagesInflight: Error = Error::Paho(ffi::MQTTASYNC_MAX_MESSAGES_INFLIGHT);

/// An invalid UTF-8 string has been detected.
pub const BadUtfString: Error = Error::Paho(ffi::MQTTASYNC_BAD_UTF8_STRING);

/// A NULL parameter has been supplied when this is invalid.
pub const NullParameter: Error = Error::Paho(ffi::MQTTASYNC_NULL_PARAMETER);

/// The topic has been truncated (the topic string includes embedded NULL characters).
/// String functions will not access the full topic.
/// Use the topic length value to access the full topic.
pub const TopicNameTruncated: Error = Error::Paho(ffi::MQTTASYNC_TOPICNAME_TRUNCATED);

/// A structure parameter does not have the correct eyecatcher and version number.
pub const BadStructure: Error = Error::Paho(ffi::MQTTASYNC_BAD_STRUCTURE);

/// A qos parameter is not 0, 1 or 2
pub const BadQos: Error = Error::Paho(ffi::MQTTASYNC_BAD_QOS);

/// All 65535 MQTT msgids are being used
pub const NoMoreMsgids: Error = Error::Paho(ffi::MQTTASYNC_NO_MORE_MSGIDS);

/// The request is being discarded when not complete
pub const OperationIncomplete: Error = Error::Paho(ffi::MQTTASYNC_OPERATION_INCOMPLETE);

/// No more messages can be buffered
pub const MaxBufferedMessages: Error = Error::Paho(ffi::MQTTASYNC_MAX_BUFFERED_MESSAGES);

/// Attempting SSL connection using non-SSL version of library
pub const SslNotSupported: Error = Error::Paho(ffi::MQTTASYNC_SSL_NOT_SUPPORTED);

/// Protocol prefix in serverURI must be "tcp://", "ssl://", "ws://", or "wss://"
/// The TLS enabled prefixes (ssl, wss) are only valid when using the AAL/TLS
/// version of the library.
pub const BadProtocol: Error = Error::Paho(ffi::MQTTASYNC_BAD_PROTOCOL);

/// Using an option for a different version of MQTT than the client or
/// connection is currently using.
pub const BadMqttOption: Error = Error::Paho(ffi::MQTTASYNC_BAD_MQTT_OPTION);

/// Call not applicable to the current version of MQTT in use.
pub const WrongMqttVersion: Error = Error::Paho(ffi::MQTTASYNC_WRONG_MQTT_VERSION);

/// Zero-length will topics not supported.
pub const ZeroLenWillTopic: Error = Error::Paho(ffi::MQTTASYNC_0_LEN_WILL_TOPIC);
