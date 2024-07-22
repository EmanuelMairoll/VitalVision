use slog::{Logger, o, Drain};
use slog_async::Async;
use slog_term::{FullFormat, TermDecorator};

pub fn create_logger(for_module: String) -> Logger {
    let decorator = TermDecorator::new().build();
    let drain = FullFormat::new(decorator)
        .use_utc_timestamp()  // Use UTC timestamp
        .use_original_order() // Maintain the order of log fields as declared
        .build()
        .fuse();
    let async_drain = Async::new(drain).build().fuse();
    let logger = Logger::root(async_drain, o!("component" => "VVCore", "module" => for_module));
    logger
}