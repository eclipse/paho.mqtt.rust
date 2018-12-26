// paho-mqtt/src/token.rs
// This file is part of the Eclipse Paho MQTT Rust Client library.

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

//! The Token module for the Paho MQTT Rust client library.
//!
//! Asynchronous operations return a `Token` that is a type of future. It
//! can be used to determine if an operation has completed, block and wait
//! for the operation to complete, and obtain the final result.
//! For example, you can start a connection, do something else, and then
//! wait for the connection to complete.
//!
//! The Token object implements the Future trait, and thus can be used and
//! combined with any other Rust futures.
//!

use std::{str, ptr};
use std::time::Duration;
use std::sync::{Arc, Mutex, Condvar};
use std::ffi::{CStr};
use std::os::raw::{c_void};

use ffi;

use async_client::AsyncClient;
use message::Message;
use errors::{MqttResult, /*MqttError,*/ ErrorKind};

/////////////////////////////////////////////////////////////////////////////
// Token

/// Callback for the token on successful completion
pub type SuccessCallback = FnMut(&AsyncClient, u16) + 'static;

/// Callback for the token on failed completion
pub type FailureCallback = FnMut(&AsyncClient, u16, i32) + 'static;

/// The result data for the token.
/// This is the guarded elements in the token which are updated by the
/// C library callback when the operation completes.
struct TokenData {
    /// Whether the async action has completed
    complete: bool,
    /// The MQTT Message ID
    msg_id: i16,
    /// The return/error code for the action (zero is success)
    ret_code: i32,
    /// The error message (if any)
    err_msg: String,
}


/// A `Token` is a mechanism for tracking the progress of an asynchronous
/// operation.
pub struct Token {
    // Mutex guards: (done, ret, msgid)
    lock: Mutex<TokenData>,
    // Signal for when the state changes
    cv: Condvar,
    // Pointer to the client that created the token.
    // This is only guaranteed valid until the end of the callback
    cli: *const AsyncClient,
    // User callback for successful completion of the async action
    on_success: Option<Box<SuccessCallback>>,
    // User callback for failed completion of the async action
    on_failure: Option<Box<FailureCallback>>,
    // The message (valid only for "delivery" tokens)
    pub(crate) msg: Option<Message>,
}

impl Token {
    /// Creates a new, unsignaled Token.
    pub fn new() -> Token {
        Token {
            lock: Mutex::new(TokenData {
                complete: false,
                msg_id: 0,
                ret_code: 0,
                err_msg: "".to_string(),
            }),
            cv: Condvar::new(),
            cli: ptr::null(),
            on_success: None,
            on_failure: None,
            msg: None
        }
    }

    /// Creates a new, un-signaled delivery Token.
    /// This is a token which tracks delivery of a message.
    pub fn from_message(msg: Message) -> Token {
        Token {
            lock: Mutex::new(TokenData {
                complete: false,
                msg_id: msg.cmsg.msgid as i16,
                ret_code: 0,
                err_msg: "".to_string(),
            }),
            cv: Condvar::new(),
            cli: ptr::null(),
            on_success: None,
            on_failure: None,
            msg: Some(msg),
        }
    }

    /// Creates a new, un-signaled Token with callbacks.
    pub fn from_client<FS,FF>(cli: *const AsyncClient,
                              success_cb: FS,
                              failure_cb: FF) -> Token
        where FS: FnMut(&AsyncClient, u16) + 'static,
              FF: FnMut(&AsyncClient, u16,i32) + 'static
    {
        Token {
            lock: Mutex::new(TokenData {
                complete: false,
                msg_id: 0,
                ret_code: 0,
                err_msg: "".to_string(),
            }),
            cv: Condvar::new(),
            cli: cli,
            on_success: Some(Box::new(success_cb)),
            on_failure: Some(Box::new(failure_cb)),
            msg: None
        }
    }

    /// Creates a new Token signaled with an error.
    pub fn from_error(rc: i32) -> Token {
        Token {
            lock: Mutex::new(TokenData {
                complete: true,
                msg_id: 0,
                ret_code: rc,
                err_msg: String::from(Token::error_msg(rc)),
            }),
            cv: Condvar::new(),
            cli: ptr::null(),
            on_success: None,
            on_failure: None,
            msg: None
        }
    }

    // Callback from the C library for when an async operation succeeds.
    pub(crate) unsafe extern "C" fn on_success(context: *mut c_void, rsp: *mut ffi::MQTTAsync_successData) {
        debug!("Token success! {:?}, {:?}", context, rsp);
        if context.is_null() {
            return
        }
        let msgid = if !rsp.is_null() { (*rsp).token as u16 } else { 0 };
        let tokptr = context as *mut Token;
        let tok = &mut *tokptr;
        tok.on_complete(&*tok.cli, msgid, 0, "".to_string());
        let _ = Arc::from_raw(tokptr);
    }

    // Callback from the C library when an async operation fails.
    pub(crate) unsafe extern "C" fn on_failure(context: *mut c_void, rsp: *mut ffi::MQTTAsync_failureData) {
        warn!("Token failure! {:?}, {:?}", context, rsp);
        if context.is_null() {
            return
        }
        let mut msgid = 0;
        let mut rc = -1;
        let mut msg = "Error".to_string();

        if !rsp.is_null() {
            msgid = (*rsp).token as u16;
            rc = if (*rsp).code == 0 { -1i32 } else { (*rsp).code as i32 };

            if !(*rsp).message.is_null() {
                if let Ok(cmsg) = CStr::from_ptr((*rsp).message).to_str() {
                    debug!("Token failure message: {:?}", cmsg);
                    msg = cmsg.to_string();
                }
            }
        }

        if msg.is_empty() {
            let emsg = Token::error_msg(rc);
            msg = emsg.to_string();
        }

        let tokptr = context as *mut Token;
        let tok = &mut *tokptr;
        // TODO: Check client null?
        tok.on_complete(&*tok.cli, msgid, rc, msg);
        let _ = Arc::from_raw(tokptr);
    }

    // Callback function to update the token when the action completes.
    pub(crate) fn on_complete(&mut self, cli: &AsyncClient, msgid: u16, rc: i32, msg: String) {
        debug!("Token completed with code: {}", rc);
        {
            let mut retv = self.lock.lock().unwrap();
            (*retv).complete = true;
            (*retv).ret_code = rc;
            (*retv).err_msg = msg;
        }
        if rc == 0 {
            if let Some(ref mut cb) = self.on_success {
                trace!("Invoking Token::on_success callback");
                cb(cli, msgid);
            }
        }
        else {
            if let Some(ref mut cb) = self.on_failure {
                trace!("Invoking Token::on_failure callback");
                cb(cli, msgid, rc);
            }
        }
        self.cv.notify_all();
    }

    // Gets the string associated with the error code from the C lib.
    pub(crate) fn error_msg(rc: i32) -> &'static str {
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
             _ => "",
        }
    }


    /// Sets the message ID for the token
    pub(crate) fn set_msgid(&self, msg_id: i16) {
        let mut retv = self.lock.lock().unwrap();
        (*retv).msg_id = msg_id;
    }


    /// Blocks the caller until the asynchronous operation has completed.
    pub fn wait(&self) -> MqttResult<()> {
        let mut retv = self.lock.lock().unwrap();

        // As long as the 'done' value inside the `Mutex` is false, we wait.
        while !(*retv).complete {
            retv = self.cv.wait(retv).unwrap();
        }

        let rc = (*retv).ret_code;
        debug!("Token completed: {}", rc);
        if rc != 0 {
            let msg = Token::error_msg(rc);
            fail!((ErrorKind::General, rc, "Error", msg));
        }
        Ok(())
    }

    /// Blocks the caller a limited amount of time waiting for the
    /// asynchronous operation to complete.
    pub fn wait_for(&self, dur: Duration) -> MqttResult<()> {
        let mut retv = self.lock.lock().unwrap();

        while !(*retv).complete {
            let result = self.cv.wait_timeout(retv, dur).unwrap();

            if result.1.timed_out() {
                fail!(::std::io::Error::new(::std::io::ErrorKind::TimedOut, "Timed out"));
            }
            retv = result.0;
        }

        let rc = (*retv).ret_code;
        debug!("Timed token completed: {}", rc);
        if rc != 0 {
            let msg = Token::error_msg(rc);
            fail!((ErrorKind::General, rc, "Error", msg));
        }

        Ok(())
    }
}

/// `Token` specificly for a message delivery operation.
/// Originally this was a distinct object, but the implementation was
/// absorbed into a standard `Token`.
pub type DeliveryToken = Token;


/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    //use super::*;

    // Makes sure than when a client is moved, the inner struct stayes at
    // the same address (on the heap) since that inner struct is used as
    // the context pointer for callbacks
    // GitHub Issue #17
    #[test]
    fn test_ok() {
        assert!(true);
    }
}

