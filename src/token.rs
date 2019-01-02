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

use std::ptr;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::ffi::CStr;
use std::os::raw::c_void;

use futures::{Future, Async};
use futures::task;
use futures::task::Task;

use ffi;

use async_client::AsyncClient;
use message::Message;
use errors;
use errors::{MqttResult, MqttError};

/////////////////////////////////////////////////////////////////////////////
// Token

/// Callback for the token on successful completion
pub type SuccessCallback = Fn(&AsyncClient, u16) + 'static;

/// Callback for the token on failed completion
pub type FailureCallback = Fn(&AsyncClient, u16, i32) + 'static;

/// The result data for the token.
/// This is the guarded elements in the token which are updated by the
/// C library callback when the operation completes.
#[derive(Debug)]
pub(crate) struct TokenData {
    /// Whether the async action has completed
    complete: bool,
    /// The MQTT Message ID
    msg_id: i16,
    /// The return/error code for the action (zero is success)
    ret_code: i32,
    /// Additional detail error message (if any)
    err_msg: Option<String>,
    /// The future task
    task: Option<Task>,
}

impl TokenData {
    /// Creates new, default token data
    pub fn new() -> TokenData {
        TokenData::default()
    }

    /// Creates token data for a specific message
    pub fn from_message_id(msg_id: i16) -> TokenData {
        TokenData {
            msg_id,
            ..TokenData::default()
        }
    }

    /// Creates a new token that is already signaled with an error.
    pub fn from_error(rc: i32) -> TokenData {
        TokenData {
            complete: true,
            ret_code: rc,
            err_msg: Some(String::from(errors::error_message(rc))),
            ..TokenData::default()
        }
    }
}

impl Default for TokenData {
    fn default() -> TokenData {
        TokenData {
            complete: false,
            msg_id: 0,
            ret_code: 0,
            err_msg: None,
            task: None,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

//#[derive(Debug)]
pub(crate) struct TokenInner {
    // Mutex guards: (done, ret, msgid)
    lock: Mutex<TokenData>,
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

impl TokenInner {
    /// Creates a new, unsignaled token.
    pub fn new() -> TokenInner {
        TokenInner::default()
    }

    /// Creates a new, un-signaled delivery token.
    /// This is a token which tracks delivery of a message.
    pub fn from_message(msg: Message) -> TokenInner {
        TokenInner {
            lock: Mutex::new(TokenData::from_message_id(msg.cmsg.msgid as i16)),
            msg: Some(msg),
            ..TokenInner::default()
        }
    }

    /// Creates a new, un-signaled Token with callbacks.
    pub fn from_client<FS,FF>(cli: *const AsyncClient,
                              success_cb: FS,
                              failure_cb: FF) -> TokenInner
        where FS: Fn(&AsyncClient, u16) + 'static,
              FF: Fn(&AsyncClient, u16,i32) + 'static
    {
        TokenInner {
            cli: cli,
            on_success: Some(Box::new(success_cb)),
            on_failure: Some(Box::new(failure_cb)),
            ..TokenInner::default()
        }
    }

    /// Creates a new token that is already signaled with an error.
    pub fn from_error(rc: i32) -> TokenInner {
        TokenInner {
            lock: Mutex::new(TokenData::from_error(rc)),
            ..TokenInner::default()
        }
    }
}

impl Default for TokenInner {
    fn default() -> Self {
        TokenInner {
            lock: Mutex::new(TokenData::new()),
            cli: ptr::null(),
            on_success: None,
            on_failure: None,
            msg: None
        }
    }
}


/////////////////////////////////////////////////////////////////////////////

/// A `Token` is a mechanism for tracking the progress of an asynchronous
/// operation.
//#[derive(Debug)]
pub struct Token {
    pub(crate) inner: Arc<TokenInner>,
}

impl Token {
    /// Creates a new, unsignaled Token.
    pub fn new() -> Token {
        Token { inner: Arc::new(TokenInner::new()) }
    }

    /// Creates a new, un-signaled delivery Token.
    /// This is a token which tracks delivery of a message.
    pub fn from_message(msg: Message) -> Token {
        Token { inner: Arc::new(TokenInner::from_message(msg)) }
    }

    /// Creates a new, un-signaled Token with callbacks.
    pub fn from_client<FS,FF>(cli: *const AsyncClient,
                              success_cb: FS,
                              failure_cb: FF) -> Token
        where FS: Fn(&AsyncClient,u16) + 'static,
              FF: Fn(&AsyncClient,u16,i32) + 'static
    {
        Token { inner: Arc::new(TokenInner::from_client(cli, success_cb, failure_cb)) }
    }

    /// Creates a new Token signaled with an error.
    pub fn from_error(rc: i32) -> Token {
        Token { inner: Arc::new(TokenInner::from_error(rc)) }
    }

    // Callback from the C library for when an async operation succeeds.
    pub(crate) unsafe extern "C" fn on_success(context: *mut c_void, rsp: *mut ffi::MQTTAsync_successData) {
        debug!("Token success! {:?}, {:?}", context, rsp);
        if context.is_null() { return }

        let tok = Token::from_raw(context);

        // TODO: Maybe compare this msgid to the one in the token?
        let msgid = if !rsp.is_null() { (*rsp).token as u16 } else { 0 };

        tok.on_complete(msgid, 0, None);
    }

    // Callback from the C library when an async operation fails.
    pub(crate) unsafe extern "C" fn on_failure(context: *mut c_void, rsp: *mut ffi::MQTTAsync_failureData) {
        warn!("Token failure! {:?}, {:?}", context, rsp);
        if context.is_null() { return }

        let tok = Token::from_raw(context);

        let mut msgid = 0;
        let mut rc = -1;
        let mut err_msg = None;

        if let Some(rsp) = rsp.as_ref() {
            msgid = rsp.token as u16;
            rc = if rsp.code == 0 { -1 } else { rsp.code as i32 };

            if !rsp.message.is_null() {
                if let Ok(cmsg) = CStr::from_ptr(rsp.message).to_str() {
                    debug!("Token failure message: {:?}", cmsg);
                    err_msg = Some(cmsg.to_string());
                }
            }
        }

        tok.on_complete(msgid, rc, err_msg);
    }

    // Callback function to update the token when the action completes.
    pub(crate) fn on_complete(&self, msgid: u16, rc: i32, err_msg: Option<String>) {
        debug!("Token completed with code: {}", rc);

        // Fire off any user callbacks

        if rc == 0 {
            if let Some(ref cb) = self.inner.on_success {
                trace!("Invoking Token::on_success callback");
                let cli = self.inner.cli;
                cb(unsafe { &*cli }, msgid);
            }
        }
        else {
            if let Some(ref cb) = self.inner.on_failure {
                trace!("Invoking Token::on_failure callback");
                let cli = self.inner.cli;
                cb(unsafe { &*cli }, msgid, rc);
            }
        }

        // Signal completion of the token

        let mut data = self.inner.lock.lock().unwrap();
        data.complete = true;
        data.ret_code = rc;
        data.err_msg = err_msg;

        // If this is none, it means that no one is waiting on
        // the future yet, so we don't need to kick it.
        if let Some(task) = data.task.as_ref() {
            task.notify();
        }
    }

    /// Consumes the `Token`, returning the inner wrapped value.
    /// This is how we generate a context pointer to send to the C lib.
    pub fn into_raw(this: Token) -> *mut c_void {
        Arc::into_raw(this.inner) as *mut c_void
    }

    /// Constructs a Token from a raw pointer to the inner structure.
    /// This is how a token is normally reconstructed from a context
    /// pointer coming back from the C lib.
    pub unsafe fn from_raw(ptr: *mut c_void) -> Token {
        let inner = Arc::from_raw(ptr as *mut TokenInner);
        Token { inner, }
    }

    /// Sets the message ID for the token
    pub(crate) fn set_msgid(&self, msg_id: i16) {
        let mut retv = self.inner.lock.lock().unwrap();
        retv.msg_id = msg_id;
    }

    /// Blocks the caller a limited amount of time waiting for the
    /// asynchronous operation to complete.
    pub fn wait_for(self, dur: Duration) -> MqttResult<()> {
        use futures_timer::FutureExt;
        let tok = self.timeout(dur);
        tok.wait()
    }
}

impl Clone for Token {
    /// Cloning a Token creates another Arc reference to
    /// the inner data on the heap.
    fn clone(&self) -> Self {
        Token { inner: self.inner.clone() }
    }
}

impl Future for Token {
    type Item = ();
    type Error = MqttError;

    /// Poll the token to see if the request has completed yet.
    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let mut data = self.inner.lock.lock().unwrap();
        let rc = data.ret_code;

        if !data.complete {
            data.task = Some(task::current());
            Ok(Async::NotReady)
        }
        else if rc == 0 {
            Ok(Async::Ready(()))
        }
        else {
            if let Some(ref err_msg) = data.err_msg {
                Err(MqttError::from((rc, err_msg.clone())))
            }
            else {
                Err(MqttError::from(rc))
            }
        }
    }
}


/// `Token` specificly for a message delivery operation.
/// Originally this was a distinct object, but the implementation was
/// absorbed into a standard `Token`.
pub type DeliveryToken = Token;


/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let tok = Token::new();
        let data = tok.inner.lock.lock().unwrap();
        assert!(!data.complete);
    }

    #[test]
    fn test_from_message() {
        const MSG_ID: i16 = 42;
        let mut msg = Message::new("hello", "Hi there", 1);
        msg.cmsg.msgid = MSG_ID as i32;

        let tok = Token::from_message(msg);
        let data = tok.inner.lock.lock().unwrap();
        assert!(!data.complete);
        assert_eq!(MSG_ID, data.msg_id);
    }

    // Created from an error code, should be complete with the right return code.
    #[test]
    fn test_from_error() {
        const ERR_CODE: i32 = -42;

        let tok = Token::from_error(ERR_CODE);
        let data = tok.inner.lock.lock().unwrap();

        assert!(data.complete);
        assert_eq!(ERR_CODE, data.ret_code);
    }

    // Cloned tokens should have the same (inner) raw address.
    #[test]
    fn test_token_clones() {
        let tok1 = Token::new();
        let tok2 = tok1.clone();

        let p1 = Token::into_raw(tok1);
        let p2 = Token::into_raw(tok2);

        assert_eq!(p1, p2);

        unsafe {
            let _ = Token::from_raw(p1);
            let _ = Token::from_raw(p2);
        }
    }
}

