use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Input file (default: stdin)
    #[arg(long = "if")]
    pub input_file: Option<String>,

    /// Output file (default: stdout)
    #[arg(long = "of")]
    pub output_file: Option<String>,

    /// Block size (in bytes, default: 512)
    #[arg(long, default_value = "512")]
    pub bs: usize,

    /// Number of blocks to copy
    #[arg(long)]
    pub count: Option<usize>,

    /// Skip blocks at start of input
    #[arg(long, default_value = "0")]
    pub skip: usize,

    /// Seek blocks at start of output
    #[arg(long, default_value = "0")]
    pub seek: usize,
}

impl Args {
    pub fn create() -> Self {
        Args::parse()
    }
}

