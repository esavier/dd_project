
use std::sync::Arc;
use std::{os::unix::fs::MetadataExt, path::PathBuf};

use bytes::BytesMut;
// use sha3::digest::core_api::Buffer;
use tokio::io::{AsyncRead, AsyncReadExt};
use blake2b_simd::Params;
use sha3::Digest;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::environment::statistics::DdContext;
use crate::io::error::IoError;
use crate::io::source::config::SourceConfig;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct DataSource {
  pub read_size: usize,
  #[derivative(Debug="ignore")]
  pub source: Box<dyn AsyncRead + Unpin + Send>,
  pub blake2b: blake2b_simd::State,
  pub sha_3_512: sha3::Sha3_512,
  pub crc32: crc32fast::Hasher,
  pub inode: u64,
  pub file_size: usize,
  pub position: usize,
  pub estimated_size: usize,
  pub sink_channel: Sender<BytesMut>,
} 



impl DataSource {
  #[tracing::instrument(level="debug", ret, err)]
  pub async fn check_permissions(source_descriptor: &PathBuf) -> Result<(), IoError> {
    let metadata = tokio::fs::metadata(source_descriptor).await.map_err(|e| IoError::InputFileOpenError(e.to_string()))?;
    if metadata.permissions().readonly() {
      return Err(IoError::InputFileNoReadPermission("Input file is read-only".to_string()));
    }
    Ok(())
  }

  #[tracing::instrument(skip(args,sink_channel), level="debug", ret, err)]
  pub async fn new(args: &SourceConfig, sink_channel: tokio::sync::mpsc::Sender<BytesMut>) -> Result<Self, IoError> {    
    Self::check_permissions(&args.input_file).await?;

    let source = tokio::fs::File::open(&args.input_file).await.map_err(|e| IoError::InputFileOpenError(e.to_string()))?;
    let metadata = tokio::fs::metadata(&args.input_file).await.map_err(|e| IoError::FileMetadataAcquireError(e.to_string()))?;

    let file_inode = metadata.ino();
    let file_size = metadata.len() as usize;
    let blake2b = Params::new().hash_length(64).to_state();
    let sha_3_512 = sha3::Sha3_512::new();
    let crc32 = crc32fast::Hasher::new();
    let position = 0;
    let estimated_size = file_size;

    let out = Self {
      read_size: args.block_size,
      source: Box::new(source),
      blake2b,
      sha_3_512,
      crc32,
      inode: file_inode,
      file_size,
      position,
      estimated_size,
      sink_channel,
    };

    Ok(out)
  }

  #[tracing::instrument(skip(sink_channel, config, dd_context), level="debug", ret, err)]
  pub async fn run(sink_channel: tokio::sync::mpsc::Sender<BytesMut>, config: SourceConfig, dd_context: Arc<Mutex<DdContext>>) -> Result<(), IoError> {
    tracing::info!("Preparing reader");  
    let mut sink = DataSource::new(&config, sink_channel).await?;

    let task = {
      dd_context.lock().await.new_task("DataSource").await
    };

    let statistics = {
      dd_context.lock().await.read_statistics.clone()
    };

    tracing::debug!("Spawning source thread");
    tokio::spawn(async move {
      task.lock().await.change_state(crate::environment::statistics::TaskStatus::Running);
      let mut statistics = statistics.lock().await;
      let notifications = dd_context.lock().await.notifications.clone();
      statistics.init();
  
      tracing::debug!("Reporting readiness");
      tracing::debug!("Started reading data");
      loop {
        let mut buf = BytesMut::with_capacity(sink.read_size);
        task.lock().await.ping();
        match sink.source.read_buf(&mut buf).await {
          Ok(bytes) => {
            tracing::debug!("read {} bytes", bytes);
            match bytes {
              0 => {
                assert!(buf.is_empty());
                sink.sink_channel.send(buf).await.unwrap(); // sebd empty buffer.
                task.lock().await.complete(0);
                notifications.notify_waiters();
                sink.sink_channel.closed().await; // we wait for the sink to finish.
                break;
              },
              _ => {
                statistics.add_read(bytes.try_into().unwrap());
                task.lock().await.ping();
                sink.sink_channel.send(buf).await.unwrap();
              }
            }
          },
          Err(e) => {
            tracing::error!("Error reading data: {}", e);
            task.lock().await.fail(-2);
            statistics.add_error();
            notifications.notify_waiters();
            break;
          }
        }
        
      }
    });
    Ok(())
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  #[tokio::test]
  async fn test_data_source_new() {
    let source_config = SourceConfig {
      input_file: PathBuf::from("Cargo.toml"),
      buffer_size: 512,
      block_size: 512,
    };

    let (sender,_receiver) = tokio::sync::mpsc::channel(1);
    let source = DataSource::new(&source_config, sender).await.unwrap();
    assert_ne!(source.file_size, 0);
    assert_eq!(source.position, 0);
    assert_ne!(source.estimated_size, 0);
    assert_ne!(source.inode, 0);
  }

  #[tokio::test]
  async fn test_data_source_check_permissions() {
    let source_config = SourceConfig {
      input_file: PathBuf::from("Cargo.toml"),
      buffer_size: 512,
      block_size: 512,
    };

    let (sender,_receiver) = tokio::sync::mpsc::channel(1);
    let source = DataSource::new(&source_config, sender).await.unwrap();

    assert!(source.file_size > 0);
    assert!(source.position == 0);
    assert!(source.estimated_size > 0);
    assert!(source.file_size == source.estimated_size);
    assert!(source.inode > 0);
  }
}
