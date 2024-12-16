#![allow(unused_imports)]

use std::sync::mpsc::{RecvError, TryRecvError};
use std::thread::sleep;
use std::{os::unix::fs::MetadataExt, path::PathBuf};
use bytes::BytesMut;
use tokio::io::{sink, AsyncWrite, AsyncWriteExt};
use blake2b_simd::Params;
use sha3::Digest;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::sync::{Mutex, Semaphore};
use tokio::io::AsyncReadExt;
use std::sync::Arc;
use std::fs::Metadata;
use crate::config;
use crate::environment::statistics::{self, DdContext};
use crate::io::error::IoError;
use crate::io::sink::config::SinkConfig;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct DataSink {
  pub write_size: usize,
  pub blake2b: blake2b_simd::State,
  pub sha_3_512: sha3::Sha3_512,
  pub crc32: crc32fast::Hasher,
  pub inode: u64,
  pub file_size: usize,
  pub position: usize,
  #[derivative(Debug="ignore")]
  pub sink: Box<dyn AsyncWrite + Unpin + Send>,
  pub metadata: Option<Metadata>,
  pub estimated_size: usize,
  pub source_channel: Receiver<BytesMut>,
}



impl DataSink { 
  #[tracing::instrument(level="debug", ret, err)]
  pub async fn check_permissions(sink_descriptor: &PathBuf) -> Result<(), IoError> {
    if ! sink_descriptor.exists() {
      tokio::fs::File::create(sink_descriptor).await.map_err(|e| IoError::InputFileOpenError(e.to_string()))?;
    }
    let metadata: std::fs::Metadata = tokio::fs::metadata(sink_descriptor).await.map_err(|e| IoError::InputFileOpenError(e.to_string()))?;
    if metadata.permissions().readonly() {
      return Err(IoError::InputFileNoReadPermission("Input file is read-only".to_string()));
    }
    Ok(())
  }

  #[tracing::instrument(skip(args, receiver), level="debug", ret, err)]
  pub async fn new(args: &SinkConfig, receiver: Receiver<BytesMut>) -> Result<Self, IoError> {    
    Self::check_permissions(&args.output_file).await?;
    let file = tokio::fs::OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(&args.output_file)
      .await
      .map_err(|e| IoError::InputFileOpenError(e.to_string()))?;

    let metadata = tokio::fs::metadata(&args.output_file).await.map_err(|e| IoError::FileMetadataAcquireError(e.to_string()))?;
    let file_inode = metadata.ino();
    let file_size = metadata.len() as usize;
    let blake2b = Params::new().hash_length(64).to_state();
    let sha_3_512 = sha3::Sha3_512::new();
    let crc32 = crc32fast::Hasher::new();
    let position = 0;
    let estimated_size = file_size;

    Ok(DataSink {
      write_size: args.block_size,
      sink: Box::new(file),
      blake2b,
      sha_3_512,
      crc32,
      inode: file_inode,
      file_size,
      position,
      metadata: Some(metadata),
      estimated_size,
      source_channel: receiver,
    })
  }

  // start a consumer thread
  #[tracing::instrument(skip(source_channel, config, dd_context), level="debug", ret, err)]
  pub async fn run(source_channel: tokio::sync::mpsc::Receiver<BytesMut>, config: SinkConfig, dd_context: Arc<Mutex<DdContext>>) -> Result<(), IoError> {
    tracing::info!("Preparing to write data");    
    let task =  {
      dd_context.lock().await.new_task("DataSink").await
    };
    let statistics = {
      dd_context.lock().await.write_statistics.clone()
    };
    statistics.lock().await.init();

    tracing::debug!("Spawning sink thread");
    tokio::spawn(async move {
      let mut data_sink: DataSink = DataSink::new(&config, source_channel).await.unwrap();
      let main_notifications = dd_context.lock().await.main_notifications.clone();
      let sink_notification = dd_context.lock().await.sink_notifications.clone();
      let source_notification = dd_context.lock().await.source_notifications.clone();
      task.lock().await.change_state(statistics::TaskStatus::Running);

      tracing::info!("Reporting readiness");
      tracing::info!("Started writing data");
      loop {
        tracing::debug!("Waiting for data");
        match data_sink.source_channel.try_recv() {
          Ok(v) => {
            tracing::debug!("Received data");
            if v.is_empty() {
              tracing::warn!("Received empty data, exiting");
              // empty data, exit
              data_sink.sink.shutdown().await.unwrap();
              sink_notification.notified().await;
              break;
            } else {
              tracing::debug!("Writing packet of {} bytes", v.len());
              data_sink.sink.write_all(&v).await.unwrap();
              // data_sink.sink.flush().await.unwrap();
              data_sink.position += v.len();
              task.lock().await.ping();
              // notification.notified().await;
            }
          },
          Err(e) => {
            match e {
              tokio::sync::mpsc::error::TryRecvError::Empty => {
                // tracing::debug!("No data available, waiting for notification");
                sleep(tokio::time::Duration::from_millis(10));
                continue;
              },
              tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                data_sink.source_channel.close();
                tracing::debug!("Channel closed, exiting");
                break;
              },             
            } 
          }
        }
      }
      loop {
        tracing::debug!("relying on the notification to exit");
        std::thread::sleep(tokio::time::Duration::from_millis(250));
        notification.notify_waiters();
      }
    });
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tokio::io::AsyncWriteExt;
  use tempfile::tempdir;

  #[tokio::test]
  async fn test_sink_new() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_sink_new");
    let mut file = tokio::fs::File::create(&file_path).await.unwrap();
    file.write_all("asdfasdasrd".as_bytes()).await.unwrap();

    let config = SinkConfig {
      output_file: file_path.clone(),
      block_size: 512,
      enable_hash: false,
      enable_crc32: false,
      enable_sha3: false,
      enable_blake2b: false,
    };

    let (_sink,source) = tokio::sync::mpsc::channel(1);
    let sink = DataSink::new(&config, source).await.unwrap();
    assert_eq!(sink.file_size, 11);
    assert_eq!(sink.inode, file_path.metadata().unwrap().ino());
  }
}
