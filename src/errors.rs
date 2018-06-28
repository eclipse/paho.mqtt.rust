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

use std::error;
use std::fmt;
use std::io;
use std::str::{/*from_utf8,*/ Utf8Error};
use std::convert::From;

/// The possible errors
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ErrorKind {
    /// General Failure
    General,
    /// Persistence Error
    PersistenceError,
    /// Bad QoS value
    QosError,
    /// Operation failed because of a type mismatch.
    TypeError,
    /// I/O Error
    IoError,
}

/// An MQTT Error
pub struct MqttError {
    repr: ErrorRepr,
}

#[derive(Debug)]
enum ErrorRepr {
    WithDescription(ErrorKind, i32, &'static str),
    WithDescriptionAndDetail(ErrorKind, i32, &'static str, String),
    IoError(io::Error),
}

impl From<io::Error> for MqttError {
    fn from(err: io::Error) -> MqttError {
        MqttError { repr: ErrorRepr::IoError(err) }
    }
}

impl From<Utf8Error> for MqttError {
    fn from(_: Utf8Error) -> MqttError {
        MqttError { repr: ErrorRepr::WithDescription(ErrorKind::TypeError, -1, "Invalid UTF-8") }
    }
}

impl From<(ErrorKind, &'static str)> for MqttError {
    fn from((kind, desc): (ErrorKind, &'static str)) -> MqttError {
        MqttError { repr: ErrorRepr::WithDescription(kind, -1, desc) }
    }
}

impl From<(ErrorKind, i32, &'static str)> for MqttError {
    fn from((kind, err, desc): (ErrorKind, i32, &'static str)) -> MqttError {
        MqttError { repr: ErrorRepr::WithDescription(kind, err, desc) }
    }
}

impl From<(ErrorKind, &'static str, String)> for MqttError {
    fn from((kind, desc, detail): (ErrorKind, &'static str, String)) -> MqttError {
        MqttError { repr: ErrorRepr::WithDescriptionAndDetail(kind, -1, desc, detail) }
    }
}

impl From<(ErrorKind, i32, &'static str, String)> for MqttError {
    fn from((kind, err, desc, detail): (ErrorKind, i32, &'static str, String)) -> MqttError {
        MqttError { repr: ErrorRepr::WithDescriptionAndDetail(kind, err, desc, detail) }
    }
}

/// MQTT Errors implement the std::error::Error trait
impl error::Error for MqttError {
    /// A short description of the error.
    /// This should not contain newlines or explicit formatting.
    fn description(&self) -> &str {
        match self.repr {
            ErrorRepr::WithDescription(_, _, desc) => desc,
            ErrorRepr::WithDescriptionAndDetail(_, _, desc, _) => desc,
            ErrorRepr::IoError(ref err) => err.description(),
        }
    }

    /// The lower-level cause of the error, if any.
    fn cause(&self) -> Option<&error::Error> {
        match self.repr {
            ErrorRepr::IoError(ref err) => Some(err as &error::Error),
            _ => None,
        }
    }
}

impl fmt::Display for MqttError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.repr {
            ErrorRepr::WithDescription(_, _err, desc) => desc.fmt(f),
            ErrorRepr::WithDescriptionAndDetail(_, _err, desc, ref detail) => {
                desc.fmt(f)?;
                f.write_str(": ")?;
                detail.fmt(f)
            }
            ErrorRepr::IoError(ref err) => err.fmt(f),
        }
    }
}

impl fmt::Debug for MqttError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

pub const PERSISTENCE_ERROR: MqttError = MqttError {
    repr: ErrorRepr::WithDescription(ErrorKind::PersistenceError, -2, "Persistence Error"),
};

/// Generic result for the entire public API
pub type MqttResult<T> = Result<T, MqttError>;


