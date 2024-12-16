use std::{collections::HashMap};

use blake2b_simd::Hash;
use bytes::{BytesMut};
use serde::{Deserialize, Serialize};
// use tokio::sync::Notify;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ItcMessageKind {
  // thread asks for something
  Request,
  // thread response for something
  Response,
  // thread sends notification, i.e. checkpoint reached, error, thread death etc,
  Notice,
  // thread sends control message, i.e. pause, stop, resume, reconfigure etc,
  Control,
  // free-form data message, advised to be in messagepack format and include common schema
  Data,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ItcMessageError {
  Unknown(String),
  Invalid(String),
  ActivelyDropped(String),
  Timeout(String),
  Unauthorized(String),
  Internal(String),
  Generic(String),
}

pub enum ItcMessageBusError {
  Unknown(String),
  SpotTaken(String),
  Timeout(String),
  Unauthorized(String),
  Internal(String),
  Generic(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItcMessage {
  sender_id: u64, // 261
  sender_name: Option<String>,
  data: Option<BytesMut>,
  kind: ItcMessageKind,

  // do we expect a response?
  message_response_uuid: Option<uuid::Uuid>,
  // should we hint for specific response kind?
  message_response_kind: Option<ItcMessageKind>,
}

pub struct TaskBus {
  task_map: HashMap<String, u64>,
  senders HashMap<u64, tokio::sync::mpsc::Sender<ItcMessage>>,
  tasks: Vec<Option<TaskState>>,
  pub capacity: u64,
}

pub struct TaskStatus {
  my_id: u64,
  my_name: Option<String>,

  // task state
  state: TaskState,
  last_update: hifitime::Epoch,

  // timestamps
  started: hifitime::Epoch,
  completed: Option<hifitime::Epoch>,
  worktime: hifitime::Duration,
}

pub struct InterThreadMessageBus {
  name_mapping: HashMap<String, u64>,
  notifications: HashMap<u64, tokio::sync::Notify>,
  send_channels: HashMap<u64, tokio::sync::mpsc::Sender<ItcMessage>>,
}

impl InterThreadMessageBus {
  pub async fn try_registering(&mut self, name: String, id: u64) -> Result<u64, ItcMessageError> {

  }
}

impl TaskStatus {
  #[must_use]
  pub async fn send_itc_message(
    &mut self, 
    channel_name: String, 
    kind: ItcMessageKind, 
    data: Option<BytesMut>, 
    callback_uuid: Option<uuid::Uuid>,
    callback_kind: Option<ItcMessageKind>,
  ) -> Result<uuid::Uuid, ItcMessageError> {
    let channel = match self.foreign_notifications.get(&channel_name) {
      Some(channel) => channel,
    }

  }
}


impl ItcMessage {

  pub fn with_data(mut self, data: BytesMut) -> ItcMessage {
    self.data = Some(data);
    self
  }
}


pub enum TaskState {
  Uninitialized,   // Task has not been initialized
  Cancelled,       // Task has been cancelled
  Pending,         // Task is initialized, pending start
  Running,         // Task is running
  Paused,          // Task is paused
  Awaiting,        // Task is awaiting either a resource or a signal
  Completed,       // Task has completed successfully
  Failed,          // Task has failed
  Stopped,         // Task has been stopped abruptly
  Crashed,         // Task has been stopped abruptly, but crash was catched 
  Custom(String),  // Task has a custom state
  Unknown,         // Task state is unknown or was not defined
}

