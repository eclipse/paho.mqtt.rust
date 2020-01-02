// paho-mqtt/src/token.rs
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2018-2019 Frank Pagliughi <fpagliughi@mindspring.com>
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
use std::convert::Into;

use futures::{Future, Async};
use futures::task;
use futures::task::Task;
use futures_timer::FutureExt;

use ffi;

use async_client::{AsyncClient};
use types::ReasonCode;
use message::Message;
//use properties::{Properties};
use server_response::{ServerRequest, ServerResponse};
use errors;
use errors::{MqttResult, MqttError};

/////////////////////////////////////////////////////////////////////////////
// TokenData

/// Callback for the token on successful completion
pub type SuccessCallback = dyn Fn(&AsyncClient, u16) + 'static;

/// Callback for the token on failed completion
pub type FailureCallback = dyn Fn(&AsyncClient, u16, i32) + 'static;

/// Callback for the token on successful completion
pub type SuccessCallback5 = dyn Fn(&AsyncClient, u16) + 'static;

/// Callback for the token on failed completion
pub type FailureCallback5 = dyn Fn(&AsyncClient, u16, i32) + 'static;

/// The result data for the token.
/// This contains the guarded elements in the token which are updated by
/// the C library callback when the asynchronous operation completes.
#[derive(Debug,Default)]
pub(crate) struct TokenData {
    /// Whether the async action has completed
    complete: bool,
    /// The MQTT Message ID
    msg_id: i16,
    /// The return/error code for the action (zero is success)
    ret_code: i32,
    /// MQTT v5 Reason Code
    reason_code: ReasonCode,
    /// Additional detail error message (if any)
    err_msg: Option<String>,
    /// The server response (dependent on the request type)
    srvr_rsp: ServerResponse,
    /// The futures task
    task: Option<Task>,
}

impl TokenData {
    /// Creates token data for a specific message
    pub fn from_message_id(msg_id: i16) -> TokenData {
        TokenData {
            msg_id,
            ..TokenData::default()
        }
    }

    /// Creates a new token that is already signaled with a code.
    pub fn from_error(rc: i32) -> TokenData {
        TokenData {
            complete: true,
            ret_code: rc,
            err_msg: if rc != 0 {
                         Some(String::from(errors::error_message(rc)))
                     }
                     else { None },
            ..TokenData::default()
        }
    }

    /// Poll the data to see if the request has completed yet.
    fn poll(&mut self) -> Result<Async<ServerResponse>, MqttError> {
        let rc = self.ret_code;

        if !self.complete {
            self.task = Some(task::current());
            Ok(Async::NotReady)
        }
        else if rc == 0 {
            Ok(Async::Ready(self.srvr_rsp.clone()))
        }
        else {
            if let Some(ref err_msg) = self.err_msg {
                Err(MqttError::from((rc, err_msg.clone())))
            }
            else {
                Err(MqttError::from(rc))
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
// TokenInner

/// The actual data structure for an asynchronous token.
/// Instances of this are passed as the context pointer to the C library
/// to track asynchronous operations. They are kept on the heap, via Arc
/// pointers so that their addresses don't change in memory. The internal
/// `TokenData` is thread-protected by a mutex as it is updated by the
/// async callback in a thread originating in the C lib. The other fields
/// are set at creation and remain static.
//#[derive(Debug)]
pub(crate) struct TokenInner {
    /// Mutex guards: (done, ret, msgid)
    lock: Mutex<TokenData>,
    /// The client that created the token.
    cli: Option<AsyncClient>,
    /// The type of request the token is tracking
    req: ServerRequest,
    /// User callback for successful completion of the async action
    on_success: Option<Box<SuccessCallback>>,
    /// User callback for failed completion of the async action
    on_failure: Option<Box<FailureCallback>>,
    /// User callback for successful completion of the MQTT v5 async action
    on_success5: Option<Box<SuccessCallback5>>,
    /// User callback for failed completion of the MQTT v5 async action
    on_failure5: Option<Box<FailureCallback5>>,
}

impl TokenInner {
    /// Creates a new, unsignaled Token.
    pub fn new() -> Arc<TokenInner> {
        Arc::new(TokenInner::default())
    }

    /// Creates a token for a specific request type
    pub fn from_request(req: ServerRequest) -> Arc<TokenInner> {
        Arc::new(
            TokenInner {
                req,
                ..TokenInner::default()
            }
        )
    }

    /// Creates a new, un-signaled delivery Token.
    /// This is a token which tracks delivery of a message.
    pub fn from_message(msg: &Message) -> Arc<TokenInner> {
        Arc::new(
            TokenInner {
                lock: Mutex::new(TokenData::from_message_id(msg.cmsg.msgid as i16)),
                ..TokenInner::default()
            }
        )
    }

    /// Creates a new, un-signaled Token with callbacks.
    pub fn from_client<FS,FF>(cli: &AsyncClient,
                              req: ServerRequest,
                              success_cb: FS,
                              failure_cb: FF) -> Arc<TokenInner>
        where FS: Fn(&AsyncClient,u16) + 'static,
              FF: Fn(&AsyncClient,u16,i32) + 'static
    {
        Arc::new(
            TokenInner {
                cli: Some(cli.clone()),
                req,
                on_success: Some(Box::new(success_cb)),
                on_failure: Some(Box::new(failure_cb)),
                ..TokenInner::default()
            }
        )
    }

    /// Creates a new Token signaled with a return code.
    pub fn from_error(rc: i32) -> Arc<TokenInner> {
        Arc::new(
            TokenInner {
                lock: Mutex::new(TokenData::from_error(rc)),
                ..TokenInner::default()
            }
        )
    }

    // Callback from the C library for when an async operation succeeds.
    pub(crate) unsafe extern "C" fn on_success(context: *mut c_void, rsp: *mut ffi::MQTTAsync_successData) {
        debug!("Token success! {:?}, {:?}", context, rsp);
        if context.is_null() { return }

        let tok = Token::from_raw(context);

        // TODO: Maybe compare this msgid to the one in the token?
        let msgid = if !rsp.is_null() { (*rsp).token as u16 } else { 0 };

        tok.inner.on_complete(msgid, 0, None, rsp);
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

        tok.inner.on_complete(msgid, rc, err_msg, ptr::null_mut());
    }

    // Callback from the C library for when an MQTT v5 async operation succeeds.
    pub(crate) unsafe extern "C" fn on_success5(context: *mut c_void, rsp: *mut ffi::MQTTAsync_successData5) {
        debug!("Token v5 success! {:?}, {:?}", context, rsp);
        if context.is_null() { return }

        let tok = Token::from_raw(context);

        // TODO: Maybe compare this msgid to the one in the token?
        let msgid = if !rsp.is_null() { (*rsp).token as u16 } else { 0 };

        tok.inner.on_complete5(msgid, 0, None, rsp);
    }

    // Callback from the C library when an MQTT v5 async operation fails.
    pub(crate) unsafe extern "C" fn on_failure5(context: *mut c_void, rsp: *mut ffi::MQTTAsync_failureData5) {
        warn!("Token v5 failure! {:?}, {:?}", context, rsp);
        if context.is_null() { return }

        let tok = Token::from_raw(context);

        let mut msgid = 0;
        let mut rc = -1;
        let mut reason_code = ReasonCode::default();
        let mut err_msg = None;

        if let Some(rsp) = rsp.as_ref() {
            msgid = rsp.token as u16;
            rc = if rsp.code == 0 { -1 } else { rsp.code as i32 };
            reason_code = ReasonCode::from_code(rsp.reasonCode);

            if !rsp.message.is_null() {
                if let Ok(cmsg) = CStr::from_ptr(rsp.message).to_str() {
                    debug!("Token failure message: {:?}", cmsg);
                    err_msg = Some(cmsg.to_string());
                }
            }
        }

        debug!("Token completed with code: {}", rc);

        // Fire off any user callbacks

        if let Some(ref cli) = tok.inner.cli {
            if let Some(ref cb) = tok.inner.on_failure {
                trace!("Invoking TokenInner::on_failure callback");
                cb(cli, msgid, rc);
            }
        }

        // Signal completion of the token

        let mut data = tok.inner.lock.lock().unwrap();
        data.complete = true;
        data.ret_code = rc;
        data.reason_code = reason_code;
        data.err_msg = err_msg;

        if let Some(rsp) = rsp.as_ref() {
            data.srvr_rsp = ServerResponse::from_failure5(rsp);
        }

        // If this is none, it means that no one is waiting on
        // the future yet, so we don't need to kick it.
        if let Some(task) = data.task.as_ref() {
            task.notify();
        }
    }

    // Callback function to update the token when the action completes.
    pub(crate) fn on_complete(&self, msgid: u16, rc: i32, err_msg: Option<String>,
                              rsp: *mut ffi::MQTTAsync_successData) {
        debug!("Token completed with code: {}", rc);

        // Fire off any user callbacks

        if let Some(ref cli) = self.cli {
            if rc == 0 {
                if let Some(ref cb) = self.on_success {
                    trace!("Invoking TokenInner::on_success callback");
                    cb(cli, msgid);
                }
            }
            else {
                if let Some(ref cb) = self.on_failure {
                    trace!("Invoking TokenInner::on_failure callback");
                    cb(cli, msgid, rc);
                }
            }
        }

        // Signal completion of the token

        let mut data = self.lock.lock().unwrap();
        data.complete = true;
        data.ret_code = rc;
        data.err_msg = err_msg;

        // Get the response from the server, if any.
        debug!("Expecting server response for: {:?}", self.req);
        unsafe {
            if let Some(rsp) = rsp.as_ref() {
                data.srvr_rsp = ServerResponse::from_success(self.req, rsp);
            }
        }
        debug!("Got response: {:?}", data.srvr_rsp);

        // If this is none, it means that no one is waiting on
        // the future yet, so we don't need to kick it.
        if let Some(task) = data.task.as_ref() {
            task.notify();
        }
    }

    // Callback function to update the token when the action completes.
    pub(crate) fn on_complete5(&self, msgid: u16, rc: i32, err_msg: Option<String>,
                                rsp: *mut ffi::MQTTAsync_successData5) {
        debug!("Token completed with code: {}", rc);

        // Fire off any user callbacks

        if let Some(ref cli) = self.cli {
            if rc == 0 {
                if let Some(ref cb) = self.on_success {
                    trace!("Invoking TokenInner::on_success callback");
                    cb(cli, msgid);
                }
            }
            else {
                if let Some(ref cb) = self.on_failure {
                    trace!("Invoking TokenInner::on_failure callback");
                    cb(cli, msgid, rc);
                }
            }
        }

        // Signal completion of the token

        let mut data = self.lock.lock().unwrap();
        data.complete = true;
        data.ret_code = rc;
        data.err_msg = err_msg;

        // Get the response from the server, if any.
        debug!("Expecting server response for: {:?}", self.req);
        unsafe {
            if let Some(rsp) = rsp.as_ref() {
                data.srvr_rsp = ServerResponse::from_success5(self.req, rsp);
            }
        }
        debug!("Got response: {:?}", data.srvr_rsp);

        // If this is none, it means that no one is waiting on
        // the future yet, so we don't need to kick it.
        if let Some(task) = data.task.as_ref() {
            task.notify();
        }
    }
}

impl Default for TokenInner {
    fn default() -> Self {
        TokenInner {
            lock: Mutex::new(TokenData::default()),
            cli: None,
            req: ServerRequest::None,
            on_success: None,
            on_failure: None,
            on_success5: None,
            on_failure5: None,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
// Token

/// A `Token` is a mechanism for tracking the progress of an asynchronous
/// operation.
#[derive(Clone)]
pub struct Token {
    pub(crate) inner: Arc<TokenInner>,
}

impl Token {
    /// Creates a new, unsignaled Token.
    pub fn new() -> Token {
        Token { inner: TokenInner::new() }
    }

    /// Creates a token for a specific request type
    pub fn from_request(req: ServerRequest) -> Token {
        Token { inner: TokenInner::from_request(req) }
    }

    /// Creates a new, un-signaled Token with callbacks.
    pub fn from_client<FS,FF>(cli: &AsyncClient,
                              req: ServerRequest,
                              success_cb: FS,
                              failure_cb: FF) -> Token
        where FS: Fn(&AsyncClient,u16) + 'static,
              FF: Fn(&AsyncClient,u16,i32) + 'static
    {
        Token { inner: TokenInner::from_client(cli, req, success_cb, failure_cb) }
    }

    /// Creates a new Token signaled with an error code.
    pub fn from_error(rc: i32) -> Token {
        Token { inner: TokenInner::from_error(rc) }
    }

    /// Creates a new Token signaled with a "success" return code.
    pub fn from_success() -> Token {
        Token { inner: TokenInner::from_error(ffi::MQTTASYNC_SUCCESS as i32) }
    }

    /// Constructs a Token from a raw pointer to the inner structure.
    /// This is how a token is normally reconstructed from a context
    /// pointer coming back from the C lib.
    pub(crate) unsafe fn from_raw(ptr: *mut c_void) -> Token {
        Token { inner: Arc::from_raw(ptr as *mut TokenInner) }
    }

    /// Consumes the `Token`, returning the inner wrapped value.
    pub(crate) fn into_raw(self) -> *mut c_void {
        Arc::into_raw(self.inner) as *mut c_void
    }

    /// Blocks the caller a limited amount of time waiting for the
    /// asynchronous operation to complete.
    pub fn wait_for(self, dur: Duration) -> MqttResult<ServerResponse> {
        self.timeout(dur).wait()
    }
}

unsafe impl Send for Token {}

impl Future for Token {
    type Item = ServerResponse;
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
            Ok(Async::Ready(data.srvr_rsp.clone()))
        }
        else if let Some(ref err_msg) = data.err_msg {
            Err(MqttError::from((rc, err_msg.clone())))
        }
        else {
            Err(MqttError::from(rc))
        }
    }
}

pub type ConnectToken = Token;
pub type SubscribeToken = Token;
pub type SubscribeManyToken = Token;
pub type UnsubscribeToken = Token;
pub type UnsubscribeManyToken = Token;

/////////////////////////////////////////////////////////////////////////////
// DeliveryToken

/// A `DeliveryToken` is a mechanism for tracking the progress of an
/// asynchronous message publish operation.
#[derive(Clone)]
pub struct DeliveryToken {
    pub(crate) inner: Arc<TokenInner>,
    msg: Message,
}

impl DeliveryToken {
    /// Creates a new, un-signaled delivery Token.
    /// This is a token which tracks delivery of a message.
    pub fn new(msg: Message) -> DeliveryToken {
        DeliveryToken {
            inner: TokenInner::from_message(&msg),
            msg,
        }
    }

    /// Creates a new Token signaled with a return code.
    pub fn from_error(msg: Message, rc: i32) -> DeliveryToken {
        DeliveryToken {
            inner: TokenInner::from_error(rc),
            msg,
        }
    }

    /// Sets the message ID for the token
    pub(crate) fn set_msgid(&self, msg_id: i16) {
        let mut data = self.inner.lock.lock().unwrap();
        data.msg_id = msg_id;
    }

    /// Gets the message associated with the publish token.
    pub fn message(&self) -> &Message {
        &self.msg
    }

    /// Blocks the caller a limited amount of time waiting for the
    /// asynchronous operation to complete.
    pub fn wait_for(self, dur: Duration) -> MqttResult<()> {
        self.timeout(dur).wait()
    }
}

unsafe impl Send for DeliveryToken {}

impl Into<Message> for DeliveryToken {
    fn into(self) -> Message { self.msg }
}

impl Into<Token> for DeliveryToken {
    /// Converts the delivery token into a Token
    fn into(self) -> Token {
        Token { inner: self.inner }
    }
}

impl Future for DeliveryToken {
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
        else if let Some(ref err_msg) = data.err_msg {
            Err(MqttError::from((rc, err_msg.clone())))
        }
        else {
            Err(MqttError::from(rc))
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

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

        let tok = DeliveryToken::new(msg);
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

    // Determine that a token can be sent across threads and signaled.
    // As long as it compiles, this indicates that Token implements the Send
    // trait.
    // TODO: This would likely deadlock on an error. Consider something that
    // would timeout on error, instead of hanging forever.
    #[test]
    fn test_token_send() {
        let tok = Token::new();
        let tok2 = tok.clone();

        let thr = thread::spawn(move || {
            tok.wait()
        });

        tok2.inner.on_complete(0, 0, None, ptr::null_mut());
        let _ = thr.join().unwrap();
    }
}

