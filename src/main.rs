pub mod config;
pub mod io;
pub mod environment;
pub mod logger;
pub mod taskstate;

use std::sync::Arc;
use config::Args;
use tokio::sync::Mutex;

use crate::io::source::config::SourceConfig;
use crate::io::sink::config::SinkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init_subscriber()?;

    let args = Args::create();
    let sink_cfg = SinkConfig::from(&args);
    let source_cfg = SourceConfig::from(&args);
    let (sink_channel, source_channel) = tokio::sync::mpsc::channel(10);
    let global_state = Arc::new(Mutex::new(environment::statistics::DdContext::new()));
    
    let main_notifications = global_state.lock().await.main_notifications.clone();
    let _sender = sink_channel.clone(); // keep so that the channel is not dropped

    io::source::core::DataSource::run(sink_channel, source_cfg, global_state.clone()).await?;
    io::sink::core::DataSink::run(source_channel, sink_cfg, global_state.clone()).await?;
    
    loop{
        {
            tracing::info!("Waiting for main_notifications");
            main_notifications.notified().await;
            tracing::info!("Notified");
        }
        global_state.lock().await.are_tasks_pending().await;
        if global_state.lock().await.are_tasks_pending().await {
            println!("Tasks are pending, waiting for them to complete");
            continue;
        } else {
            println!("All tasks are complete");
            break;
        }
    }

    println!("\n\nprinting out statistics");
    global_state.lock().await.display_statistics().await;
    global_state.lock().await.display_tasks().await;
    for (k,v) in global_state.lock().await.task_status.iter() {
        println!("Task: {}", k);
        v.lock().await.display();
    }
    Ok(())
}
