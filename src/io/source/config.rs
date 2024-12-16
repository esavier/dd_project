use std::path::PathBuf;

#[derive(derivative::Derivative)]
#[derivative(Default)]
pub struct SourceConfig {
  pub input_file: PathBuf,
  #[derivative(Default(value = "512"))]
  pub buffer_size: usize,
  #[derivative(Default(value = "512"))]
  pub block_size: usize,
}

impl SourceConfig {
  pub fn new() -> Self {
    SourceConfig::default()
  }
}

impl From<&crate::config::Args> for SourceConfig {
  fn from(args: &crate::config::Args) -> Self {
    SourceConfig {
      input_file: PathBuf::from(args.input_file.clone().unwrap_or("".to_string())),
      buffer_size: args.bs,
      block_size: args.bs,
    }
  }
}