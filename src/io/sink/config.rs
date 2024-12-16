use std::path::PathBuf;


#[derive(derivative::Derivative)]
#[derivative(Default)]
pub struct SinkConfig {
  pub output_file: PathBuf,
  #[derivative(Default(value = "512"))]
  pub block_size: usize,
  #[derivative(Default(value = "false"))]
  pub enable_hash: bool,
  #[derivative(Default(value = "false"))]
  pub enable_crc32: bool,
  #[derivative(Default(value = "false"))]
  pub enable_sha3: bool,
  #[derivative(Default(value = "false"))]
  pub enable_blake2b: bool,
}

impl SinkConfig {
  pub fn new() -> Self {
    SinkConfig::default()
  }
}

impl From<&crate::config::Args> for SinkConfig {
  fn from(args: &crate::config::Args) -> Self {
    SinkConfig {
      output_file: PathBuf::from(args.output_file.clone().unwrap_or("".to_string())),
      block_size: args.bs,
      enable_hash: false,
      enable_crc32: false,
      enable_sha3: false,
      enable_blake2b: false,
    }
  }
}