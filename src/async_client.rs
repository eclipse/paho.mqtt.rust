// async_client.rs
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017 Frank Pagliughi <fpagliughi@mindspring.com>
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

use std::str;
use std::{ptr, slice};	// mem
use std::time::Duration;
use std::sync::{Arc, Mutex, Condvar};
use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char, c_int};

use ffi;

use connect_options::{ConnectOptions};
use message::{Message};
use errors::{MqttResult, /*MqttError,*/ ErrorKind};

/////////////////////////////////////////////////////////////////////////////
// Token

// Callback for the token on successful completion
pub type SuccessCallback = FnMut(&AsyncClient, u16) + 'static;

// Callback for the token on failed completion
pub type FailureCallback = FnMut(&AsyncClient, u16, i32) + 'static;

/// The result data for the token.
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
	/// Creates a new, unsignalled Token.
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

	/// Creates a new, unsignalled delivery Token.
	/// This ia a token which tracks delivery of a message.
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

	/// Creates a new, unsignalled Token with callbacks.
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

	// Callback from the C library for when an async operation succeeds.
	unsafe extern "C" fn on_success(context: *mut c_void, rsp: *mut ffi::MQTTAsync_successData) {
		println!("Token success! {:?}, {:?}", context, rsp);
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
		println!("Token failure! {:?}, {:?}", context, rsp);
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
					println!("Token failure message: {:?}", cmsg);
					msg = cmsg.to_string();
				}
			}
		}

		if msg.is_empty() {
			let emsg = match rc {
				/*MQTTASYNC_FAILURE*/ -1 => "General failure",
				/*MQTTASYNC_PERSISTENCE_ERROR*/ -2 => "Persistence error",
				/*MQTTASYNC_DISCONNECTED*/ -3 => "Client disconnected",
				/*MQTTASYNC_MAX_MESSAGES_INFLIGHT*/ -4 => "Maximum inflight messages",
				/*MQTTASYNC_BAD_UTF8_STRING*/ -5 => "Bad UTF8 string",
				/*MQTTASYNC_NULL_PARAMETER*/ -6 => "NULL Parameter",
				/*MQTTASYNC_TOPICNAME_TRUNCATED*/ -7 => "Topic name truncated",
				/*MQTTASYNC_BAD_STRUCTURE*/ -8 => "Bad structure",
				/*MQTTASYNC_BAD_QOS*/ -9 => "Bad QoS",
				/*MQTTASYNC_NO_MORE_MSGIDS*/ -10 => "No more message ID's",
				/*MQTTASYNC_OPERATION_INCOMPLETE*/ -11 => "Operation incomplete",
				/*MQTTASYNC_MAX_BUFFERED_MESSAGES*/ -12 => "Max buffered messages",
				/*MQTTASYNC_SSL_NOT_SUPPORTED*/ -13 => "SSL not supported by Paho C library",
				 _ => "",
			};
			msg = emsg.to_string();
		}

		let tokptr = context as *mut Token;
		let tok = &mut *tokptr;
		// TODO: Check client null
		tok.on_complete(&*tok.cli, msgid, rc, msg);
		let _ = Arc::from_raw(tokptr);
	}

	// Callback function to update the token when the action completes.
	fn on_complete(&mut self, cli: &AsyncClient, msgid: u16, rc: i32, msg: String) {
		println!("Token completed with code: {}", rc);
		{
			let mut retv = self.lock.lock().unwrap();
			(*retv).complete = true;
			(*retv).ret_code = rc;
			(*retv).err_msg = msg;
		}
		if rc == 0 {
			if let Some(ref mut cb) = self.on_success {
				println!("Invoking Token::on_success callback");
				cb(cli, msgid);
			}
		}
		else {
			if let Some(ref mut cb) = self.on_failure {
				println!("Invoking Token::on_failure callback");
				cb(cli, msgid, rc);
			}
		}
		self.cv.notify_all();
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
		println!("Token completed: {}", rc);
		// TODO: Get real error result & message
		if rc != 0 { 
			let msg = (*retv).err_msg.clone();
			fail!((ErrorKind::General, rc, "Error", msg)); 
		}
		Ok(())
	}

	/// Blocks the caller a limited amount of time waiting for the
	/// asynchronous operation to complete.
	pub fn wait_timeout(&self, dur: Duration) -> MqttResult<()> {
		let mut retv = self.lock.lock().unwrap();

		while !(*retv).complete {
			let result = self.cv.wait_timeout(retv, dur).unwrap();

			if result.1.timed_out() {
				fail!(::std::io::Error::new(::std::io::ErrorKind::TimedOut, "Timed out"));
			}
			retv = result.0;
		}

		let rc = (*retv).ret_code;
		println!("Timed token completed: {}", rc);
		// TODO: Get real error result & message
		if rc != 0 { 
			let msg = (*retv).err_msg.clone();
			fail!((ErrorKind::General, rc, "Error", msg)); 
		}

		Ok(())
	}
}

type DeliveryToken = Token;

/////////////////////////////////////////////////////////////////////////////
// AsynClient

// User callback type for when the connection is lost from the broker.
pub type ConnectionLostCallback = FnMut(&mut AsyncClient) + 'static;

// User callback signature for when subscribed messages are received.
pub type MessageArrivedCallback = FnMut(&AsyncClient, Message) + 'static;

// The context provided for the client callbacks.
// Note that the Paho C library maintains a single void* context pointer
// shared between all of the callbacks. We could use just a pointer to the
// client and retrieve the callbacks from there, but that would require
// every callback to synchronize data access from the callback.
struct CallbackContext
{
	on_connection_lost: Option<Box<ConnectionLostCallback>>,
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
	//username: CString,
	//pasword: CString,
}

impl AsyncClient {
	unsafe extern "C" fn on_connected(context: *mut c_void, rsp: *mut ffi::MQTTAsync_successData) {
		println!("Connected! {:?}, {:?}", context, rsp);
	}

	// Low-level callback for when the connection is lost.
	unsafe extern "C" fn on_connection_lost(context: *mut c_void,
											_cause: *mut c_char) {
		println!("\nConnection lost. Context: {:?}", context);
		if !context.is_null() {
			let cli = context as *mut AsyncClient;
			let mut cbctx = (*cli).callback_context.lock().unwrap();

			if let Some(ref mut cb) = (*cbctx).on_connection_lost {
				println!("Invoking connection lost callback");
				cb(&mut *cli);
			}
		}
	}

	// Low-level callback for when a message arrives from the broker.
	unsafe extern "C" fn on_message_arrived(context: *mut c_void,
											topic_name: *mut c_char,
											topic_len: c_int,
											mut cmsg: *mut ffi::MQTTAsync_message) -> c_int {
		println!("\nMessage arrived. Context: {:?}, topic: {:?} len {:?} cmsg: {:?}: {:?}",
				 context, topic_name, topic_len, cmsg, *cmsg);

		if !context.is_null() {
			let cli = context as *mut AsyncClient;
			let mut cbctx = (*cli).callback_context.lock().unwrap();

			if let Some(ref mut cb) = (*cbctx).on_message_arrived {

				let len = topic_len as usize;
				let tp = str::from_utf8(slice::from_raw_parts(topic_name as *mut u8, len)).unwrap();
				println!("Topic Slice: {}", tp);
				let topic = CString::new(tp).unwrap();
				println!("Topic: {:?}", topic);

				let msg = Message::from_c_parts(topic, &*cmsg);

				println!("Invoking message callback");
				cb(&*cli, msg);
			}
		}

		ffi::MQTTAsync_freeMessage(&mut cmsg);	// as *mut *mut ffi::MQTTAsync_message);
		ffi::MQTTAsync_free(topic_name as *mut c_void);
		1
	}

	/// Creates a new MQTT client which can connect to an MQTT broker.
	///
	/// # Arguments
	///
	/// * `server_uri` The address of the MQTT broker.
	/// * `client_id` The unique name of the client. if this is empty, the
	///		the broker will assign a unique name.
	///
	pub fn new(server_uri: &str, client_id: &str) -> AsyncClient {
		let mut cli = AsyncClient {
			handle: ptr::null_mut(),
			opts: Mutex::new(ConnectOptions::new()),
			callback_context: Mutex::new(CallbackContext {
				on_connection_lost: None,
				on_message_arrived: None,
			}),
			server_uri: CString::new(server_uri).unwrap(),
			client_id: CString::new(client_id).unwrap(),
		};

		let ret;
		unsafe {
			ret = ffi::MQTTAsync_create(&mut cli.handle as *mut *mut c_void,
										cli.server_uri.as_ptr(),
										cli.client_id.as_ptr(),
										0, ptr::null_mut()) as i32;
		}

		println!("Create result: {}", ret);
		println!("handle: {:?}", cli.handle);

		cli
	}

	/// Connects to an MQTT broker using the specified connect options.
	///
	/// # Arguments
	///
	/// * `opts` The connect options
	pub fn connect(&self, opts: ConnectOptions) -> Arc<Token> {
		println!("Connecting handle: {:?}", self.handle);

		let tok = Arc::new(Token::new());
		let tokcb = tok.clone();

		let mut lkopts = self.opts.lock().unwrap();
		*lkopts = opts;
		(*lkopts).copts.onSuccess = Some(Token::on_success);
		(*lkopts).copts.onFailure = Some(Token::on_failure);
		(*lkopts).copts.context = Arc::into_raw(tokcb) as *mut c_void;

		let ts = unsafe { CStr::from_ptr((*(*lkopts).copts.ssl).trustStore) };
		unsafe {
			println!("Connect Trust Store: [{:?}] {:?}", (*(*lkopts).copts.ssl).trustStore, ts);
		}

		let ret = unsafe {
			ffi::MQTTAsync_connect(self.handle, &(*lkopts).copts)
		};

		println!("Connection result: {}", ret);
		//if ret == 0 { Ok(tok) } else { Err(From::from((ErrorKind::General, ret, "Connection error"))) }
		tok
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
		println!("Connecting handle: {:?}", self.handle);
		println!("\nConnect opts: {:?}", opts);
		unsafe {
			if !opts.copts.will.is_null() {
				println!("\nWill: {:?}", *(opts.copts.will));
			}
		}

		let t = Token::from_client(self as *const _, success_cb, failure_cb);
		let tok = Arc::new(t);
		let tokcb = tok.clone();

		opts.copts.onSuccess = Some(Token::on_success);
		opts.copts.onFailure = Some(Token::on_failure);
		opts.copts.context = Arc::into_raw(tokcb) as *mut c_void;;
		println!("\nConnect opts: {:?}", opts);
		{
			let mut lkopts = self.opts.lock().unwrap();
			*lkopts = opts.clone();
		}

		let ret = unsafe {
			ffi::MQTTAsync_connect(self.handle, &opts.copts)
		};

		println!("Connection result: {}", ret);
		tok
	}

	/// Attempts to reconnect to the broker.
	/// This can only be called after a connection was initially made or 
	/// attempted. It will retry with the same connect options.
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
	pub fn disconnect(&mut self) -> Arc<Token> {
		println!("Disconnecting");

		let tok = Arc::new(Token::new());
		let tokcb = tok.clone();

		let mut opts = ffi::MQTTAsync_disconnectOptions::default();
		opts.onSuccess = Some(Token::on_success);
		opts.context = Arc::into_raw(tokcb) as *mut c_void;

		let ret;
		unsafe {
			ret = ffi::MQTTAsync_disconnect(self.handle, &opts);
		}
		println!("Disconnection result: {}", ret);
		tok
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
	/// 	function or a closure.
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
	/// 	function or a closure.
	pub fn set_message_callback<F>(&mut self, cb: F)
		where F: FnMut(&AsyncClient,Message) + 'static
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
	pub fn publish(&mut self, msg: Message) -> Arc<DeliveryToken> {
		println!("Publish: {:?}", msg);

		let tok = Arc::new(DeliveryToken::from_message(msg));
		let tokcb = tok.clone();

		let mut opts = ffi::MQTTAsync_responseOptions::default();
		opts.onSuccess = Some(Token::on_success);
		opts.context = Arc::into_raw(tokcb) as *mut c_void;

		let ret;

		unsafe {
			let msg = tok.msg.as_ref().unwrap();
			ret = ffi::MQTTAsync_sendMessage(self.handle, msg.topic.as_ptr(), &msg.cmsg, &mut opts);
			println!("Publish result: {}", ret);
		}

		if ret != 0 {
			// TODO: Handle the error
			println!("Send error: {}", ret);
		}

		tok.set_msgid(opts.token as i16);
		tok
	}

	/// Subscribes to a single topic.
	///
	/// # Arguments
	///
	/// `topic` The topic name
	/// `qos` The quality of service requested for messages
	pub fn subscribe(&self, topic: &str, qos: i32) -> Arc<Token> {
		println!("Subscribe to '{}' @ QOS {}", topic, qos);

		let tok = Arc::new(DeliveryToken::new());
		let tokcb = tok.clone();

		let mut opts = ffi::MQTTAsync_responseOptions::default();
		opts.onSuccess = Some(Token::on_success);
		opts.context = Arc::into_raw(tokcb) as *mut c_void;

		let topic = CString::new(topic).unwrap();

		let ret = unsafe {
			ffi::MQTTAsync_subscribe(self.handle, topic.as_ptr(), qos, &mut opts)
		};

		println!("Subscribe result: {}", ret);

		if ret != 0 {
			// TODO: Handle the error
			println!("Subscribe error: {}", ret);
		}

		tok
	}
}

