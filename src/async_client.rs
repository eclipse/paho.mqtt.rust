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
//! This presents an asynchronous API that is similar to the other Paho MQTT
//! clients, but but uses Token objects that implement the Futures trait, so
//! can be used in much more flexible ways than the other language clients.
//!
//! Asynchronous operations return a `Token` that is a type of future. It
//! can be used to determine if an operation has completed, block and wait
//! for the operation to complete, and obtain the final result.
//! For example, you can start a connection, do something else, and then
//! wait for the connection to complete.
//!
//! ```
//! extern crate futures;
//! extern crate paho_mqtt as mqtt;
//!
//! use futures::future::Future;
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
use std::sync::{Mutex};
use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char, c_int};

use ffi;

use create_options::{CreateOptions,PersistenceType};
use connect_options::ConnectOptions;
use disconnect_options::{DisconnectOptions,DisconnectOptionsBuilder};
use response_options::ResponseOptions;
use message::Message;
use token::{Token, DeliveryToken};
use client_persistence::UserPersistence;
use errors;
use errors::{MqttResult, ErrorKind};
use string_collection::{StringCollection};

/////////////////////////////////////////////////////////////////////////////
// AsynClient

/// User callback type for when the connection is lost from the broker.
pub type ConnectionLostCallback = FnMut(&AsyncClient) + 'static;

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
struct InnerAsyncClient {
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
    // The user persistence (if any)
    user_persistence: Option<Box<UserPersistence>>
}

/// An asynchronous MQTT connection client.
pub struct AsyncClient {
    inner: Box<InnerAsyncClient>,
}

impl AsyncClient {
    // Low-level callback from the C library when the client is connected.
    // We currently don't use this for anything. Rather connection
    // completion is tracked through a token.
    unsafe extern "C" fn on_connected(context: *mut c_void, rsp: *mut ffi::MQTTAsync_successData) {
        debug!("Connected! {:?}, {:?}", context, rsp);
    }

    // Low-level callback from the C library when the connection is lost.
    // We pass the call on to the handler registered with the client, if any.
    unsafe extern "C" fn on_connection_lost(context: *mut c_void,
                                            _cause: *mut c_char) {
        warn!("Connection lost. Context: {:?}", context);
        if !context.is_null() {
            let pcli: Box<InnerAsyncClient> = Box::from_raw(context as *mut _);
            let cli = AsyncClient { inner: pcli };
            {
                let mut cbctx = cli.inner.callback_context.lock().unwrap();

                if let Some(ref mut cb) = cbctx.on_message_arrived {
                    trace!("Invoking disconnect message callback");
                    cb(&cli, None);
                }

                if let Some(ref mut cb) = cbctx.on_connection_lost {
                    trace!("Invoking connection lost callback");
                    cb(&cli);
                }
            }
            let _ = Box::into_raw(cli.inner);
        }
    }

    // Low-level callback from the C library when a message arrives from the broker.
    // We pass the call on to the handler registered with the client, if any.
    unsafe extern "C" fn on_message_arrived(context: *mut c_void,
                                            topic_name: *mut c_char,
                                            topic_len: c_int,
                                            mut cmsg: *mut ffi::MQTTAsync_message) -> c_int {
        debug!("Message arrived. Context: {:?}, topic: {:?} len {:?} cmsg: {:?}: {:?}",
               context, topic_name, topic_len, cmsg, *cmsg);

        if !context.is_null() {
            let pcli: Box<InnerAsyncClient> = Box::from_raw(context as *mut _);
            let cli = AsyncClient { inner: pcli };
            {
                let mut cbctx = cli.inner.callback_context.lock().unwrap();

                if let Some(ref mut cb) = cbctx.on_message_arrived {
                    let len = topic_len as usize;
                    let topic = if len == 0 {
                        // Zero-len topic means it's a NUL-terminated C string
                        CStr::from_ptr(topic_name).to_owned()
                    }
                    else {
                        // If we get a len for the topic, then there's no NUL terminator.
                        // TODO: Handle UTF-8 error(s)
                        let tp = str::from_utf8(slice::from_raw_parts(topic_name as *mut u8, len)).unwrap();
                        CString::new(tp).unwrap()
                    };
                    let msg = Message::from_c_parts(topic, &*cmsg);

                    trace!("Invoking message callback");
                    cb(&cli, Some(msg));
                }
            }
            let _ = Box::into_raw(cli.inner);
        }

        ffi::MQTTAsync_freeMessage(&mut cmsg);
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

        let mut cli = InnerAsyncClient {
            handle: ptr::null_mut(),
            opts: Mutex::new(ConnectOptions::new()),
            callback_context: Mutex::new(CallbackContext {
                on_connection_lost: None,
                on_message_arrived: None,
            }),
            server_uri: CString::new(opts.server_uri).unwrap(),
            client_id: CString::new(opts.client_id).unwrap(),
            user_persistence: None,
        };

        let (ptype, pptr) = match opts.persistence {
            PersistenceType::User(cli_persist) => {
                let mut user_persistence = Box::new(UserPersistence::new(cli_persist));
                let pptr = &mut user_persistence.copts as *mut _ as *mut c_void;
                cli.user_persistence = Some(user_persistence);
                (ffi::MQTTCLIENT_PERSISTENCE_USER, pptr)
            },
            PersistenceType::File => (ffi::MQTTCLIENT_PERSISTENCE_DEFAULT, ptr::null_mut()),
            PersistenceType::None => (ffi::MQTTCLIENT_PERSISTENCE_NONE, ptr::null_mut()),
        };

        debug!("Creating client with persistence: {}", ptype);

        let rc = unsafe {
            ffi::MQTTAsync_createWithOptions(&mut cli.handle as *mut *mut c_void,
                                             cli.server_uri.as_ptr(),
                                             cli.client_id.as_ptr(),
                                             ptype as c_int,
                                             pptr,
                                             &mut opts.copts) as i32
        };

        if rc != 0 {
            warn!("Create result: {}", rc);
            fail!((ErrorKind::General, rc, errors::error_message(rc)));
        }
        debug!("AsyncClient handle: {:?}", cli.handle);

        let cli = AsyncClient {
            inner: Box::new(cli),
        };

        Ok(cli)
    }

    /// Connects to an MQTT broker using the specified connect options.
    ///
    /// # Arguments
    ///
    /// * `opts` The connect options
    ///
    pub fn connect<T>(&self, opt_opts: T) -> Token
        where T: Into<Option<ConnectOptions>>
    {
        if let Some(opts) = opt_opts.into() {
            debug!("Connecting handle: {:?}", self.inner.handle);
            debug!("Connect options: {:?}", opts);

            let tok = Token::new();

            let mut lkopts = self.inner.opts.lock().unwrap();
            *lkopts = opts;
            lkopts.set_token(tok.clone());

            let rc = unsafe {
                ffi::MQTTAsync_connect(self.inner.handle, &lkopts.copts)
            };

            if rc != 0 {
                let _ = unsafe { Token::from_raw(lkopts.copts.context) };
                Token::from_error(rc)
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
                                         failure_cb: FF) -> Token
        where FS: Fn(&AsyncClient,u16) + 'static,
              FF: Fn(&AsyncClient,u16,i32) + 'static
    {
        debug!("Connecting handle: {:?}", self.inner.handle);
        debug!("Connect opts: {:?}", opts);
        unsafe {
            if !opts.copts.will.is_null() {
                debug!("Will: {:?}", *(opts.copts.will));
            }
        }

        let tok = Token::from_client(self as *const _, success_cb, failure_cb);
        opts.set_token(tok.clone());

        debug!("Connect opts: {:?}", opts);
        {
            let mut lkopts = self.inner.opts.lock().unwrap();
            *lkopts = opts.clone();
        }

        let rc = unsafe {
            ffi::MQTTAsync_connect(self.inner.handle, &opts.copts)
        };

        if rc != 0 {
            let _ = unsafe { Token::from_raw(opts.copts.context) };
            Token::from_error(rc)
        }
        else { tok }
    }

    /// Attempts to reconnect to the broker.
    /// This can only be called after a connection was initially made or
    /// attempted. It will retry with the same connect options.
    ///
    pub fn reconnect(&self) -> Token {
        let connopts = {
            let lkopts = self.inner.opts.lock().unwrap();
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
                                           failure_cb: FF) -> Token
        where FS: Fn(&AsyncClient,u16) + 'static,
              FF: Fn(&AsyncClient,u16,i32) + 'static
    {
        let connopts = {
            let lkopts = self.inner.opts.lock().unwrap();
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
    pub fn disconnect<T>(&self, opt_opts: T) -> Token
            where T: Into<Option<DisconnectOptions>>
    {
        if let Some(mut opts) = opt_opts.into() {
            debug!("Disconnecting");

            let tok = Token::new();
            opts.set_token(tok.clone());

            let rc = unsafe {
                ffi::MQTTAsync_disconnect(self.inner.handle, &opts.copts)
            };

            if rc != 0 {
                let _ = unsafe { Token::from_raw(opts.copts.context) };
                Token::from_error(rc)
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
    pub fn disconnect_after(&self, timeout: Duration) -> Token {
        let disconn_opts = DisconnectOptionsBuilder::new()
                                .timeout(timeout).finalize();
        self.disconnect(disconn_opts)
    }

    /// Determines if this client is currently connected to an MQTT broker.
    pub fn is_connected(&self) -> bool {
        unsafe {
            ffi::MQTTAsync_isConnected(self.inner.handle) != 0
        }
    }

    /// Sets the callback for when the connection is lost with the broker.
    ///
    /// # Arguments
    ///
    /// * `cb` The callback to register with the library. This can be a
    ///     function or a closure.
    pub fn set_connection_lost_callback<F>(&mut self, cb: F)
        where F: FnMut(&AsyncClient) + 'static
    {
        // A pointer to the inner client will serve as the callback context
        let ctx: &InnerAsyncClient = &self.inner;

        // This should be protected by a mutex if we'll have a thread-safe client
        {
            let mut cbctx = self.inner.callback_context.lock().unwrap();
            (*cbctx).on_connection_lost = Some(Box::new(cb));
        }

        unsafe {
            ffi::MQTTAsync_setCallbacks(self.inner.handle,
                                        ctx as *const _ as *mut c_void,
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
        // A pointer to the inner client will serve as the callback context
        let ctx: &InnerAsyncClient = &self.inner;

        // This should be protected by a mutex if we'll have a thread-safe client
        {
            let mut cbctx = self.inner.callback_context.lock().unwrap();
            (*cbctx).on_message_arrived = Some(Box::new(cb));
        }

        unsafe {
            ffi::MQTTAsync_setCallbacks(self.inner.handle,
                                        ctx as *const _ as *mut c_void,
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
    pub fn publish(&self, msg: Message) -> DeliveryToken {
        debug!("Publish: {:?}", msg);

        let tok = DeliveryToken::from_message(msg);
        let mut rsp_opts = ResponseOptions::new(tok.clone());

        let rc = unsafe {
            let msg = tok.inner.msg.as_ref().unwrap();
            ffi::MQTTAsync_sendMessage(self.inner.handle, msg.topic.as_ptr(),
                                       &msg.cmsg, &mut rsp_opts.copts)
        };

        if rc != 0 {
            let _ = unsafe { Token::from_raw(rsp_opts.copts.context) };
            Token::from_error(rc)
        }
        else {
            tok.set_msgid(rsp_opts.copts.token as i16);
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
    pub fn subscribe<S>(&self, topic: S, qos: i32) -> Token
        where S: Into<String>
    {
        let tok = DeliveryToken::new();
        let mut rsp_opts = ResponseOptions::new(tok.clone());
        let topic = CString::new(topic.into()).unwrap();

        debug!("Subscribe to '{:?}' @ QOS {}", topic, qos);

        let rc = unsafe {
            ffi::MQTTAsync_subscribe(self.inner.handle, topic.as_ptr(), qos, &mut rsp_opts.copts)
        };

        if rc != 0 {
            let _ = unsafe { Token::from_raw(rsp_opts.copts.context) };
            Token::from_error(rc)
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
    pub fn subscribe_many<T>(&self, topics: &[T], qos: &[i32]) -> Token
        where T: AsRef<str>
    {
        // TOOD: Make sure topics & qos are same length (or use min)
        let tok = Token::new();
        let mut rsp_opts = ResponseOptions::new(tok.clone());
        let topics = StringCollection::new(topics);

        debug!("Subscribe to '{:?}' @ QOS {:?}", topics, qos);

        let rc = unsafe {
            ffi::MQTTAsync_subscribeMany(self.inner.handle,
                                         topics.len() as c_int,
                                         topics.as_c_arr_mut_ptr(),
                                         // C lib takes mutable QoS ptr, but doesn't mutate
                                         mem::transmute(qos.as_ptr()),
                                         &mut rsp_opts.copts)
        };

        if rc != 0 {
            let _ = unsafe { Token::from_raw(rsp_opts.copts.context) };
            Token::from_error(rc)
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
    pub fn unsubscribe<S>(&self, topic: S) -> Token
        where S: Into<String>
    {
        let tok = Token::new();
        let mut rsp_opts = ResponseOptions::new(tok.clone());
        let topic = CString::new(topic.into()).unwrap();

        debug!("Unsubscribe from '{:?}'", topic);

        let rc = unsafe {
            ffi::MQTTAsync_unsubscribe(self.inner.handle, topic.as_ptr(),
                                       &mut rsp_opts.copts)
        };

        if rc != 0 {
            let _ = unsafe { Token::from_raw(rsp_opts.copts.context) };
            Token::from_error(rc)
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
    pub fn unsubscribe_many<T>(&self, topics: &[T]) -> Token
        where T: AsRef<str>
    {
        let tok = Token::new();
        let mut rsp_opts = ResponseOptions::new(tok.clone());
        let topics = StringCollection::new(topics);

        debug!("Unsubscribe from '{:?}'", topics);

        let rc = unsafe {
            ffi::MQTTAsync_unsubscribeMany(self.inner.handle,
                                           topics.len() as c_int,
                                           topics.as_c_arr_mut_ptr(),
                                           &mut rsp_opts.copts)
        };

        if rc != 0 {
            let _ = unsafe { Token::from_raw(rsp_opts.copts.context) };
            Token::from_error(rc)
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
    pub fn start_consuming(&mut self) -> std::sync::mpsc::Receiver<Option<Message>> {
        use std::sync::mpsc;
        use std::sync::mpsc::{Sender, Receiver};

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

    /// Creates a futures stream for consuming messages.
    pub fn get_stream(&mut self, buffer_sz: usize) -> futures::sync::mpsc::Receiver<Option<Message>> {
        use futures::sync::mpsc;

        let (mut tx, rx) = mpsc::channel(buffer_sz);

        self.set_message_callback(move |_,msg| {
            if let Err(err) = tx.try_send(msg) {
                if err.is_full() {
                    warn!("Stream losing messages");
                }
                else {
                    error!("Stream error: {:?}", err);
                    // TODO: Can we do anything here?
                }
            }
        });

        rx
    }
}

impl Drop for InnerAsyncClient {
    /// Drops the client by closing dpen all the underlying, dependent objects
    fn drop(&mut self) {
        // Destroy the underlying C client.
        if !self.handle.is_null() {
            unsafe {
                ffi::MQTTAsync_destroy(&mut self.handle as *mut *mut c_void);
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
        let mut cli = InnerAsyncClient {
            handle: ptr::null_mut(),
            opts: Mutex::new(ConnectOptions::new()),
            callback_context: Mutex::new(CallbackContext {
                on_connection_lost: None,
                on_message_arrived: None,
            }),
            server_uri: CString::new(self.server_uri.clone()).unwrap(),
            client_id: CString::new(self.client_id.clone()).unwrap(),
            user_persistence: None,
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
        AsyncClient {
            inner: Box::new(cli),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use create_options::{CreateOptionsBuilder};
    use futures::Future;

    // Makes sure than when a client is moved, the inner struct stayes at
    // the same address (on the heap) since that inner struct is used as
    // the context pointer for callbacks
    // GitHub Issue #17
    #[test]
    fn test_context() {
        let mut cli = AsyncClient::new("tcp://localhost:1883").unwrap();
        cli.set_message_callback(|_, _| {});

        // Get a context pointer to the inner struct
        let pctx = {
            let ctx: &InnerAsyncClient = &cli.inner;
            ctx as *const _ as *mut c_void
        };

        // Move the client, then get a context pointer to inner
        let new_cli = cli;
        let new_pctx = {
            let ctx: &InnerAsyncClient = &new_cli.inner;
            ctx as *const _ as *mut c_void
        };

        // They should match (inner didn't move)
        assert_eq!(pctx, new_pctx);
    }
    
    #[test]
    fn test_create_async_client() {
        let client = AsyncClient::new("tcp://localhost:1883");
        assert!(client.is_ok(), "Error in creating simple async client, do you have a running MQTT server on localhost:1883?");
    }

    #[test]
    fn test_async_client_with_client_id() {
        println!("With client id");
        let options = CreateOptionsBuilder::new().client_id("test1").finalize();
        let client = AsyncClient::new(options);
        assert!(client.is_ok(), "Error in creating async client with client_id");
        let tok = client.unwrap().connect(None);
        match tok.wait() {
            Ok(_) => (),
            Err(e) => println!("(Error) {}", e)
        }
    }
}

