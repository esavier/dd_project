/*
Copyright (C) 2023 ErgLabs <dev@erglabs.org>.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};
pub fn init_subscriber() -> Result<(), Box<dyn std::error::Error>> {
    // let file_appender = tracing_appender::rolling::daily("logs",
    // "netstream.log");
    // todo:esavier switch for file logging
    // let subscriber = tracing_subscriber::registry()
    //     .with(EnvFilter::from_default_env())
    //     .with(
    //         fmt::Layer::new()
    //             .with_file(false)
    //             .with_line_number(true)
    //             .with_ansi(false)
    //             .with_writer(file_appender),
    //     );
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            fmt::Layer::default()
                .with_target(true)
                .with_thread_names(true)
                .with_ansi(true)
                .with_line_number(true)
                .with_file(true)
                .with_thread_ids(true),
        );
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global logger instance");
  
    Ok(())
}
