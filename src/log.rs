use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::tools::fs;

/// Discard is a writer that discards all data written to it.
struct Discard;

impl std::io::Write for Discard {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

static mut _LOG_WORKER_GUARD: Vec<WorkerGuard> = vec![];

pub async fn logging_initialize() {
    //-> Vec<WorkerGuard> {

    #![allow(static_mut_refs)]
    unsafe {
        if _LOG_WORKER_GUARD.len() >= 2 {
            return;
        }
    }

    let rebit = crate::conf::rebit().read().expect("conf::rebit is not initialized");
    let app_name = rebit.name.clone();
    let log_conf = match &rebit.log {
        None => Default::default(),
        Some(log) => log.clone(),
    };

    let (nonblocking, _guard) = tracing_appender::non_blocking(Discard {});
    let (console, console_reload) =
        tracing_subscriber::reload::Layer::new(tracing_subscriber::fmt::layer().with_ansi(false).with_writer(nonblocking.clone()));

    let (persist, persist_reload) =
        tracing_subscriber::reload::Layer::new(tracing_subscriber::fmt::layer().with_ansi(false).with_writer(nonblocking.clone()));

    let mut guards: Vec<WorkerGuard> = vec![];
    if log_conf.console {
        let (writer, guard) = tracing_appender::non_blocking(std::io::stdout());
        guards.push(guard);
        console_reload
            .reload(tracing_subscriber::fmt::layer().with_writer(writer).with_ansi(true))
            .expect("console reload failed");
    }

    let logs_dir = log_conf.dirs.trim();
    if logs_dir.len() > 0 {
        let is = fs::Is(logs_dir.to_string());
        if !is.dir().await {
            panic!("log dir is not a directory: {}", logs_dir);
        }
        let prefix = format!("{}_rings.log", app_name);

        let (writer, guard) = tracing_appender::non_blocking(tracing_appender::rolling::daily(logs_dir, prefix));

        guards.push(guard);
        persist_reload
            .reload(tracing_subscriber::fmt::layer().with_writer(writer).with_ansi(false))
            .expect("persist reload failed");
    }

    let directives: &str = &log_conf.level;
    let filter = tracing_subscriber::EnvFilter::new(directives);
    tracing_subscriber::registry().with(console).with(persist).with(filter).init();

    unsafe {
        _LOG_WORKER_GUARD.extend(guards);
    }
}

#[allow(unused)]
fn _sanitize_string(s: &str) -> String {
    s.chars().filter(|&c| !c.is_control() || c.is_whitespace()).collect()
}
