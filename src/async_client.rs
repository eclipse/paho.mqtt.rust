// paho-mqtt/src/async_client.rs
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

//! The Asynchronous client module for the Paho MQTT Rust client library.
//!
//! Currently this presents an asynchronous API that is similar to the
//! other Paho MQTT clients, but is not based on any other Rust async
//! library like mio or tokio.
//!
//! Asynchronous operations return a `Token` that is a type of future. It
//! can be used to determine if an operation has completed, block and wait
//! for the operation to complete, and obtain the final result.
//! For example, you can start a connection, do something else, and then
//! wait for the connection to complete.
//!
//! ```
//! extern crate paho_mqtt as mqtt;
//!
//! let cli = mqtt::AsyncClient::new("tcp://localhost:1883").unwrap();
//!
//! // Start an async operation and get the token for it.
//! let tok = cli.connect(mqtt::ConnectOptions::new());
//!
//! // ...do something else...
//!
//! // Wait for the async operation to complete.
//! tok.wait().unwrap();
//! ```

use std::str;
use std::{ptr, slice, mem};
use std::time::Duration;
use std::sync::{Arc, Mutex, Condvar};
use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char, c_int};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use ffi;

use create_options::{CreateOptions,PersistenceType};
use connect_options::ConnectOptions;
use disconnect_options::{DisconnectOptions,DisconnectOptionsBuilder};
use message::Message;
use client_persistence::{/*ClientPersistence,*/ ClientPersistenceBridge};
use errors::{MqttResult, /*MqttError,*/ ErrorKind};
use string_collection::{StringCollection};

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
    msg: Option<Message>,
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
    unsafe extern "C" fn on_success(context: *mut c_void, rsp: *mut ffi::MQTTAsync_successData) {
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
    unsafe extern "C" fn on_failure(context: *mut c_void, rsp: *mut ffi::MQTTAsync_failureData) {
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
    fn on_complete(&mut self, cli: &AsyncClient, msgid: u16, rc: i32, msg: String) {
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
    fn error_msg(rc: i32) -> &'static str {
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
    fn set_msgid(&self, msg_id: i16) {
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
// AsynClient

/// User callback type for when the connection is lost from the broker.
pub type ConnectionLostCallback = FnMut(&mut AsyncClient) + 'static;

/// User callback signature for when subscribed messages are received.
pub type MessageArrivedCallback = FnMut(&AsyncClient, Option<Message>) + 'static;

// The context provided for the client callbacks.
// Note that the Paho C library maintains a single void* context pointer
// shared between all of the callbacks. We could use just a pointer to the
// client and retrieve the callbacks from there, but that would require
// every callback to synchronize data access from the callback.
struct CallbackContext
{
    /// Callback for when the client loses connection to the server.
    on_connection_lost: Option<Box<ConnectionLostCallback>>,
    /// Callback for when a message arrives from the server.
    on_message_arrived: Option<Box<MessageArrivedCallback>>,
}

/// An asynchronous MQTT connection client.
pub struct AsyncClient {
    // The handle to the Paho C client
    handle: ffi::MQTTAsync,
    // The options for connecting to the broker
    opts: Mutex<ConnectOptions>,
    // The context to give to the C callbacks
    callback_context: Mutex<CallbackContext>,
    // The server URI
    server_uri: CString,
    // The MQTT client ID name
    client_id: CString,
    // Raw pointer to the user persistence (if any)
    // This is a consumed box, and should be dropped manually
    persistence_ptr: *mut ffi::MQTTClient_persistence,
}

impl AsyncClient {
    unsafe extern "C" fn on_connected(context: *mut c_void, rsp: *mut ffi::MQTTAsync_successData) {
        debug!("Connected! {:?}, {:?}", context, rsp);
    }

    // Low-level callback for when the connection is lost.
    unsafe extern "C" fn on_connection_lost(context: *mut c_void,
                                            _cause: *mut c_char) {
        warn!("Connection lost. Context: {:?}", context);
        if !context.is_null() {
            let cli = context as *mut AsyncClient;
            let mut cbctx = (*cli).callback_context.lock().unwrap();

            if let Some(ref mut cb) = (*cbctx).on_message_arrived {
                trace!("Invoking disconnect message callback");
                cb(&*cli, None);
            }

            if let Some(ref mut cb) = (*cbctx).on_connection_lost {
                trace!("Invoking connection lost callback");
                cb(&mut *cli);
            }
        }
    }

    // Low-level callback for when a message arrives from the broker.
    unsafe extern "C" fn on_message_arrived(context: *mut c_void,
                                            topic_name: *mut c_char,
                                            topic_len: c_int,
                                            mut cmsg: *mut ffi::MQTTAsync_message) -> c_int {
        debug!("Message arrived. Context: {:?}, topic: {:?} len {:?} cmsg: {:?}: {:?}",
               context, topic_name, topic_len, cmsg, *cmsg);

        if !context.is_null() {
            let cli = context as *mut AsyncClient;
            let mut cbctx = (*cli).callback_context.lock().unwrap();

            if let Some(ref mut cb) = (*cbctx).on_message_arrived {
                let len = topic_len as usize;
                let topic = if len == 0 {
                    info!("Got a zero-length topic");
                    CStr::from_ptr(topic_name).to_owned()
                }
                else {
                    // TODO: Handle UTF-8 error(s)
                    let tp = str::from_utf8(slice::from_raw_parts(topic_name as *mut u8, len)).unwrap();
                    CString::new(tp).unwrap()
                };
                let msg = Message::from_c_parts(topic, &*cmsg);

                trace!("Invoking message callback");
                cb(&*cli, Some(msg));
            }
        }

        ffi::MQTTAsync_freeMessage(&mut cmsg);  // as *mut *mut ffi::MQTTAsync_message);
        ffi::MQTTAsync_free(topic_name as *mut c_void);
        1
    }
    /// Creates a new MQTT client which can connect to an MQTT broker.
    ///
    /// # Arguments
    ///
    /// `opts` The create options for the client.
    ///
    pub fn new<T>(opts: T) -> MqttResult<AsyncClient>
        where T: Into<CreateOptions>
    {
        let mut opts = opts.into();

        // TODO: Don't unwrap() CStrings. Return error instead.

        let mut cli = AsyncClient {
            handle: ptr::null_mut(),
            opts: Mutex::new(ConnectOptions::new()),
            callback_context: Mutex::new(CallbackContext {
                on_connection_lost: None,
                on_message_arrived: None,
            }),
            server_uri: CString::new(opts.server_uri).unwrap(),
            client_id: CString::new(opts.client_id).unwrap(),
            persistence_ptr: ptr::null_mut(),
        };

        let (ptype, usrptr) = match opts.persistence {
            PersistenceType::User(persist) => (ffi::MQTTCLIENT_PERSISTENCE_USER,
                                               Box::into_raw(persist) as *mut c_void),
            PersistenceType::File => (ffi::MQTTCLIENT_PERSISTENCE_DEFAULT, ptr::null_mut()),
            PersistenceType::None => (ffi::MQTTCLIENT_PERSISTENCE_NONE, ptr::null_mut()),
        };

        debug!("Creating client with persistence: {}, {:?}", ptype, usrptr);

        if !usrptr.is_null() {
            // TODO: The bridge should return boxed persistence given uptr
            let persistence = Box::new(ffi::MQTTClient_persistence {
                context: usrptr,
                popen: Some(ClientPersistenceBridge::on_open),
                pclose: Some(ClientPersistenceBridge::on_close),
                pput: Some(ClientPersistenceBridge::on_put),
                pget: Some(ClientPersistenceBridge::on_get),
                premove: Some(ClientPersistenceBridge::on_remove),
                pkeys: Some(ClientPersistenceBridge::on_keys),
                pclear: Some(ClientPersistenceBridge::on_clear),
                pcontainskey: Some(ClientPersistenceBridge::on_contains_key),
            });

            // Note that the C library does NOT keep a copy of this persistence
            // store structure. We must keep a copy alive for as long as the
            // client remains active.
            cli.persistence_ptr = Box::into_raw(persistence);
        }

        let rc = unsafe {
            ffi::MQTTAsync_createWithOptions(&mut cli.handle as *mut *mut c_void,
                                             cli.server_uri.as_ptr(),
                                             cli.client_id.as_ptr(),
                                             ptype as c_int,
                                             cli.persistence_ptr as *mut c_void,
                                             &mut opts.copts) as i32
        };

        if rc != 0 {
            warn!("Create result: {}", rc);
            fail!((ErrorKind::General, rc, Token::error_msg(rc)));
        }
        debug!("AsyncClient handle: {:?}", cli.handle);
        Ok(cli)
    }

    /// Connects to an MQTT broker using the specified connect options.
    ///
    /// # Arguments
    ///
    /// * `opts` The connect options
    ///
    pub fn connect<T>(&self, opt_opts: T) -> Arc<Token>
        where T: Into<Option<ConnectOptions>>
    {
        if let Some(opts) = opt_opts.into() {
            debug!("Connecting handle: {:?}", self.handle);
            debug!("Connect options: {:?}", opts);

            let tok = Arc::new(Token::new());
            let tokcb = tok.clone();

            let mut lkopts = self.opts.lock().unwrap();
            *lkopts = opts;
            (*lkopts).copts.onSuccess = Some(Token::on_success);
            (*lkopts).copts.onFailure = Some(Token::on_failure);
            (*lkopts).copts.context = Arc::into_raw(tokcb) as *mut c_void;

            let rc = unsafe {
                ffi::MQTTAsync_connect(self.handle, &(*lkopts).copts)
            };

            if rc != 0 {
                let _ = unsafe { Arc::from_raw((*lkopts).copts.context as *mut Token) };
                Arc::new(Token::from_error(rc))
            }
            else { tok }
        }
        else {
            self.connect(Some(ConnectOptions::default()))
        }
    }


    /// Connects to an MQTT broker using the specified connect options.
    ///
    /// # Arguments
    ///
    /// * `opts` The connect options
    ///
    pub fn connect_with_callbacks<FS,FF>(&self,
                                         mut opts: ConnectOptions,
                                         success_cb: FS,
                                         failure_cb: FF) -> Arc<Token>
        where FS: FnMut(&AsyncClient,u16) + 'static,
              FF: FnMut(&AsyncClient,u16,i32) + 'static
    {
        debug!("Connecting handle: {:?}", self.handle);
        debug!("Connect opts: {:?}", opts);
        unsafe {
            if !opts.copts.will.is_null() {
                debug!("Will: {:?}", *(opts.copts.will));
            }
        }

        let t = Token::from_client(self as *const _, success_cb, failure_cb);
        let tok = Arc::new(t);
        let tokcb = tok.clone();

        opts.copts.onSuccess = Some(Token::on_success);
        opts.copts.onFailure = Some(Token::on_failure);
        opts.copts.context = Arc::into_raw(tokcb) as *mut c_void;;
        debug!("Connect opts: {:?}", opts);
        {
            let mut lkopts = self.opts.lock().unwrap();
            *lkopts = opts.clone();
        }

        let rc = unsafe {
            ffi::MQTTAsync_connect(self.handle, &opts.copts)
        };

        if rc != 0 {
            let _ = unsafe { Arc::from_raw(opts.copts.context as *mut Token) };
            Arc::new(Token::from_error(rc))
        }
        else { tok }
    }

    /// Attempts to reconnect to the broker.
    /// This can only be called after a connection was initially made or
    /// attempted. It will retry with the same connect options.
    ///
    pub fn reconnect(&self) -> Arc<Token> {
        let connopts = {
            let lkopts = self.opts.lock().unwrap();
            (*lkopts).clone()
        };
        self.connect(connopts)
    }

    /// Attempts to reconnect to the broker, using callbacks to signal
    /// completion.
    /// This can only be called after a connection was initially made or
    /// attempted. It will retry with the same connect options.
    ///
    /// # Arguments
    ///
    /// * `success_cb` The callback for a successful connection.
    /// * `failure_cb` The callback for a failed connection attempt.
    ///
    pub fn reconnect_with_callbacks<FS,FF>(&self,
                                           success_cb: FS,
                                           failure_cb: FF) -> Arc<Token>
        where FS: FnMut(&AsyncClient,u16) + 'static,
              FF: FnMut(&AsyncClient,u16,i32) + 'static
    {
        let connopts = {
            let lkopts = self.opts.lock().unwrap();
            (*lkopts).clone()
        };
        self.connect_with_callbacks(connopts, success_cb, failure_cb)
    }

    /// Disconnects from the MQTT broker.
    ///
    /// # Arguments
    ///
    /// `opt_opts` Optional disconnect options. Specifying `None` will use
    ///            default of immediate (zero timeout) disconnect.
    ///
    pub fn disconnect<T>(&self, opt_opts: T) -> Arc<Token>
            where T: Into<Option<DisconnectOptions>>
    {
        if let Some(mut opts) = opt_opts.into() {
            debug!("Disconnecting");

            let tok = Arc::new(Token::new());
            let tokcb = tok.clone();

            opts.copts.onSuccess = Some(Token::on_success);
            opts.copts.onFailure = Some(Token::on_failure);
            opts.copts.context = Arc::into_raw(tokcb) as *mut c_void;

            let rc = unsafe {
                ffi::MQTTAsync_disconnect(self.handle, &opts.copts)
            };

            if rc != 0 {
                let _ = unsafe { Arc::from_raw(opts.copts.context as *mut Token) };
                Arc::new(Token::from_error(rc))
            }
            else { tok }
        }
        else {
            self.disconnect(Some(DisconnectOptions::default()))
        }
    }

    /// Disconnect from the MQTT broker with a timeout.
    /// This will delay the disconnect for up to the specified timeout to
    /// allow in-flight messages to complete.
    /// This is the same as calling disconnect with options specifying a
    /// timeout.
    ///
    /// # Arguments
    ///
    /// `timeout` The amount of time to wait for the disconnect. This has
    ///           a resolution in milliseconds.
    ///
    pub fn disconnect_after(&self, timeout: Duration) -> Arc<Token> {
        let disconn_opts = DisconnectOptionsBuilder::new()
                                .timeout(timeout).finalize();
        self.disconnect(disconn_opts)
    }

    /// Determines if this client is currently connected to an MQTT broker.
    pub fn is_connected(&self) -> bool {
        unsafe {
            ffi::MQTTAsync_isConnected(self.handle) != 0
        }
    }

    /// Sets the callback for when the connection is lost with the broker.
    ///
    /// # Arguments
    ///
    /// * `cb` The callback to register with the library. This can be a
    ///     function or a closure.
    pub fn set_connection_lost_callback<F>(&mut self, cb: F)
        where F: FnMut(&mut AsyncClient) + 'static
    {
        // A pointer to self will serve as the callback context
        let self_ptr = self as *mut _ as *mut c_void;

        // This should be protected by a mutex if we'll have a thread-safe client
        {
            let mut cbctx = self.callback_context.lock().unwrap();
            (*cbctx).on_connection_lost = Some(Box::new(cb));
        }

        unsafe {
            ffi::MQTTAsync_setCallbacks(self.handle,
                                        self_ptr,
                                        Some(AsyncClient::on_connection_lost),
                                        Some(AsyncClient::on_message_arrived),
                                        None /* Delivery Complete (unused, Tokens track this) */);
        }
    }

    /// Sets the callback for when a message arrives from the broker.
    ///
    /// # Arguments
    ///
    /// * `cb` The callback to register with the library. This can be a
    ///     function or a closure.
    ///
    pub fn set_message_callback<F>(&mut self, cb: F)
        where F: FnMut(&AsyncClient, Option<Message>) + 'static
    {
        // A pointer to self will serve as the callback context
        let self_ptr = self as *mut _ as *mut c_void;

        // This should be protected by a mutex if we'll have a thread-safe client
        {
            let mut cbctx = self.callback_context.lock().unwrap();
            (*cbctx).on_message_arrived = Some(Box::new(cb));
        }

        unsafe {
            ffi::MQTTAsync_setCallbacks(self.handle,
                                        self_ptr,
                                        Some(AsyncClient::on_connection_lost),
                                        Some(AsyncClient::on_message_arrived),
                                        None /* Delivery Complete (unused, Tokens track this) */);
        }
    }

    /// Publishes a message to an MQTT broker
    ///
    /// # Arguments
    ///
    /// * `msg` The message to publish.
    ///
    pub fn publish(&self, msg: Message) -> Arc<DeliveryToken> {
        debug!("Publish: {:?}", msg);

        let tok = Arc::new(DeliveryToken::from_message(msg));
        let tokcb = tok.clone();

        let mut copts = ffi::MQTTAsync_responseOptions::default();
        copts.onSuccess = Some(Token::on_success);
        copts.context = Arc::into_raw(tokcb) as *mut c_void;

        let rc = unsafe {
            let msg = tok.msg.as_ref().unwrap();
            ffi::MQTTAsync_sendMessage(self.handle, msg.topic.as_ptr(), &msg.cmsg, &mut copts)
        };

        if rc != 0 {
            let _ = unsafe { Arc::from_raw(copts.context as *mut Token) };
            Arc::new(Token::from_error(rc))
        }
        else {
            tok.set_msgid(copts.token as i16);
            tok
        }
    }

    /// Subscribes to a single topic.
    ///
    /// # Arguments
    ///
    /// `topic` The topic name
    /// `qos` The quality of service requested for messages
    ///
    pub fn subscribe<S>(&self, topic: S, qos: i32) -> Arc<Token>
        where S: Into<String>
    {
        let tok = Arc::new(DeliveryToken::new());
        let tokcb = tok.clone();

        let mut copts = ffi::MQTTAsync_responseOptions::default();
        copts.onSuccess = Some(Token::on_success);
        copts.context = Arc::into_raw(tokcb) as *mut c_void;

        let topic = CString::new(topic.into()).unwrap();

        debug!("Subscribe to '{:?}' @ QOS {}", topic, qos);

        let rc = unsafe {
            ffi::MQTTAsync_subscribe(self.handle, topic.as_ptr(), qos, &mut copts)
        };

        if rc != 0 {
            let _ = unsafe { Arc::from_raw(copts.context as *mut Token) };
            Arc::new(Token::from_error(rc))
        }
        else { tok }
    }

    /// Subscribes to multiple topics simultaneously.
    ///
    /// # Arguments
    ///
    /// `topics` The collection of topic names
    /// `qos` The quality of service requested for messages
    ///
    pub fn subscribe_many<T>(&self, topics: &[T], qos: &[i32]) -> Arc<Token>
        where T: AsRef<str>
    {
        // TOOD: Make sure topics & qos are same length (or use min)
        let tok = Arc::new(DeliveryToken::new());
        let tokcb = tok.clone();

        let mut copts = ffi::MQTTAsync_responseOptions::default();
        copts.onSuccess = Some(Token::on_success);
        copts.context = Arc::into_raw(tokcb) as *mut c_void;

        let topics = StringCollection::new(topics);

        debug!("Subscribe to '{:?}' @ QOS {:?}", topics, qos);

        let rc = unsafe {
            ffi::MQTTAsync_subscribeMany(self.handle,
                                         topics.len() as c_int,
                                         topics.as_c_arr_mut_ptr(),
                                         // C lib takes mutable QoS ptr, but doesn't mutate
                                         mem::transmute(qos.as_ptr()),
                                         &mut copts)
        };

        if rc != 0 {
            let _ = unsafe { Arc::from_raw(copts.context as *mut Token) };
            Arc::new(Token::from_error(rc))
        }
        else { tok }
    }

    /// Unsubscribes from a single topic.
    ///
    /// # Arguments
    ///
    /// `topic` The topic to unsubscribe. It must match a topic from a
    ///         previous subscribe.
    ///
    pub fn unsubscribe<S>(&self, topic: S) -> Arc<Token>
        where S: Into<String>
    {
        let tok = Arc::new(DeliveryToken::new());
        let tokcb = tok.clone();

        let mut copts = ffi::MQTTAsync_responseOptions::default();
        copts.onSuccess = Some(Token::on_success);
        copts.context = Arc::into_raw(tokcb) as *mut c_void;

        let topic = CString::new(topic.into()).unwrap();

        debug!("Unsubscribe from '{:?}'", topic);

        let rc = unsafe {
            ffi::MQTTAsync_unsubscribe(self.handle, topic.as_ptr(), &mut copts)
        };

        if rc != 0 {
            let _ = unsafe { Arc::from_raw(copts.context as *mut Token) };
            Arc::new(Token::from_error(rc))
        }
        else { tok }
    }

    /// Unsubscribes from multiple topics simultaneously.
    ///
    /// # Arguments
    ///
    /// `topic` The topics to unsubscribe. Each must match a topic from a
    ///         previous subscribe.
    ///
    pub fn unsubscribe_many<T>(&self, topics: &[T]) -> Arc<Token>
        where T: AsRef<str>
    {
        let tok = Arc::new(DeliveryToken::new());
        let tokcb = tok.clone();

        let mut copts = ffi::MQTTAsync_responseOptions::default();
        copts.onSuccess = Some(Token::on_success);
        copts.context = Arc::into_raw(tokcb) as *mut c_void;

        let topics = StringCollection::new(topics);

        debug!("Unsubscribe from '{:?}'", topics);

        let rc = unsafe {
            ffi::MQTTAsync_unsubscribeMany(self.handle,
                                           topics.len() as c_int,
                                           topics.as_c_arr_mut_ptr(),
                                           &mut copts)
        };

        if rc != 0 {
            let _ = unsafe { Arc::from_raw(copts.context as *mut Token) };
            Arc::new(Token::from_error(rc))
        }
        else { tok }
    }

    /// Starts the client consuming messages.
    /// This starts the client receiving messages and placing them into an
    /// mpsc queue. It returns the receiving-end of the queue for the
    /// application to get the messages.
    /// This can be called at any time after the client is created, but it
    /// should be called before subscribing to any topics, otherwise messages
    /// can be lost.
    //
    pub fn start_consuming(&mut self) -> mpsc::Receiver<Option<Message>> {
        let (tx, rx): (Sender<Option<Message>>, Receiver<Option<Message>>) = mpsc::channel();

        self.set_message_callback(move |_,msg| {
            tx.send(msg).unwrap();
        });

        rx
    }

    /// Stops the client from consuming messages.
    pub fn stop_consuming(&self) {
        unimplemented!();
    }
}

impl Drop for AsyncClient {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::MQTTAsync_destroy(&mut self.handle as *mut *mut c_void);
            }
        }
        if !self.persistence_ptr.is_null() {
            unsafe {
                let context = (*self.persistence_ptr).context;
                if !context.is_null() {
                    drop(Box::from_raw(context));
                }
                drop(Box::from_raw(self.persistence_ptr));
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
//                              Builder
/////////////////////////////////////////////////////////////////////////////

/// Builder to collect the MQTT asynchronous client creation options.
pub struct AsyncClientBuilder
{
    copts: ffi::MQTTAsync_createOptions,
    server_uri: String,
    client_id: String,
    persistence_type: i32,  // TODO: Make this an enumeration
}

impl AsyncClientBuilder {
    /// Creates a new `AsyncClientBuilder`
    pub fn new() -> AsyncClientBuilder {
        AsyncClientBuilder {
            copts: ffi::MQTTAsync_createOptions::default(),
            server_uri: "".to_string(),
            client_id: "".to_string(),
            persistence_type: 0,        // 0 = Default file persistence
        }
    }

    /// Sets the address for the MQTT broker/server.
    ///
    /// # Arguments
    ///
    /// `server_uri` The address of the MQTT broker. It takes the form
    ///              <i>protocol://host:port</i>, where <i>protocol</i> must
    ///              be <i>tcp</i> or <i>ssl</i>. For <i>host</i>, you can
    ///              specify either an IP address or a host name. For instance,
    ///              to connect to a server running on the local machines with
    ///              the default MQTT port, specify <i>tcp://localhost:1883</i>.
    pub fn server_uri(&mut self, server_uri: &str) -> &mut AsyncClientBuilder {
        self.server_uri = server_uri.to_string();
        self
    }

    /// Sets the client identifier for connection to the broker.
    ///
    /// # Arguments
    ///
    /// `client_id` A unique identifier string to be passed to the broker
    ///             when the connection is made. This must be a UTF-8 encoded
    ///             string. If it is empty, the broker will create and assign
    ///             a unique name for the client.
    pub fn client_id(&mut self, client_id: &str) -> &mut AsyncClientBuilder {
        self.client_id = client_id.to_string();
        self
    }

    /// Turns default file persistence on or off.
    /// When turned on, the client will use the default, file-based,
    /// persistence mechanism. This stores information about in-flight
    /// messages in persistent storage on the file system, and provides
    /// some protection against message loss in the case of unexpected
    /// failure.
    /// When turned off, the client uses in-memory persistence. If the
    /// client crashes or system power fails, the client could lose
    /// messages.
    ///
    /// # Arguments
    ///
    /// `on` Whether to turn on file-based message persistence.
    pub fn persistence(&mut self, on: bool) -> &mut AsyncClientBuilder {
        // 0=file persistence, 1=persistence off
        self.persistence_type = if on { 0 } else { 1 };
        self
    }

    // TODO:
    // This will allow the app to specify a user-defined persistence mechanism
//  pub fn user_persistence<T: UserPersistence>(&mut self, persistence: T)
//              -> &mut AsyncClientBuilder {
//      // Setup the user persistence
//  }

    /// Enables or disables off-line buffering of out-going messages when
    /// the client is disconnected.
    ///
    /// # Arguments
    ///
    /// `on` Whether or not the application is allowed to publish messages
    ///      if the client is off-line.
    pub fn offline_buffering(&mut self, on: bool) -> &mut AsyncClientBuilder {
        self.copts.sendWhileDisconnected = if on { 1 } else { 0 };
        self
    }

    /// Enables off-line buffering of out-going messages when the client is
    /// disconnected and sets the maximum number of messages that can be
    /// buffered.
    ///
    /// # Arguments
    ///
    /// `max_buffered_msgs` The maximum number of messages that the client
    ///                     will buffer while off-line.
    pub fn max_buffered_messages(&mut self, max_buffered_messages: i32) -> &mut AsyncClientBuilder {
        self.copts.sendWhileDisconnected = 1;   // Turn it on
        self.copts.maxBufferedMessages = max_buffered_messages;
        self
    }

    /// Finalize the builder and create an asynchronous client.
    pub fn finalize(&self) -> AsyncClient {
        let mut cli = AsyncClient {
            handle: ptr::null_mut(),
            opts: Mutex::new(ConnectOptions::new()),
            callback_context: Mutex::new(CallbackContext {
                on_connection_lost: None,
                on_message_arrived: None,
            }),
            server_uri: CString::new(self.server_uri.clone()).unwrap(),
            client_id: CString::new(self.client_id.clone()).unwrap(),
            persistence_ptr: ptr::null_mut(),
        };

        // TODO We wouldn't need this if C options were immutable in call
        // to ffi:MQTTAsync:createWithOptions
        let mut copts = self.copts.clone();

        debug!("Create opts: {:?}", copts);

        let rc = unsafe {
            ffi::MQTTAsync_createWithOptions(&mut cli.handle as *mut *mut c_void,
                                             cli.server_uri.as_ptr(),
                                             cli.client_id.as_ptr(),
                                             self.persistence_type, ptr::null_mut(),
                                             &mut copts)
        };

        if rc != 0 { warn!("Create failure: {}", rc); }
        debug!("AsyncClient handle: {:?}", cli.handle);

        // TODO: This can fail. We should return a Result<AsyncClient>
        cli
    }
}

