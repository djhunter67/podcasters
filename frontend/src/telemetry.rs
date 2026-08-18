use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

#[must_use]
/// # Result
///  - `None` if the `RUST_LOG` environment variable is not set
/// # Errors
///  - If the `RUST_LOG` environment variable is set but the value is invalid
/// # Panics
///  - If the `RUST_LOG` environment variable is set but the value is invalid
pub fn get_subcriber(debug: bool) -> impl tracing::Subscriber + Send + Sync {
    let env_filter = if debug {
        String::from("info")
    } else {
        String::from("debug")
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let stdout_log = tracing_subscriber::fmt::layer().pretty();
    let subscriber = Registry::default().with(env_filter).with(stdout_log);

    let json_log = if debug {
        None
    } else {
        let json_log = tracing_subscriber::fmt::layer().compact().pretty();
        Some(json_log)
    };

    subscriber.with(json_log)
}

/// # Result
///  - `None` if the `RUST_LOG` environment variable is not set
/// # Errors
///  - If the `RUST_LOG` environment variable is set but the value is invalid
/// # Panics
///  - If the `RUST_LOG` environment variable is set but the value is invalid
pub fn init_subscriber(subscriber: impl tracing::Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
