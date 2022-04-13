use js_sys::JsString;
use web_sys::console;

pub use log::LevelFilter::*;

struct JsLog;

impl log::Log for JsLog {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }
    fn log(&self, record: &log::Record<'_>) {
        console::log_1(&JsString::from(format!("{}", record.args())));
    }
    fn flush(&self) {}
}

pub fn init(verbosity: log::LevelFilter) {
    // log to console
    fern::Dispatch::new()
        .level(verbosity)
        .format(|out, message, record| out.finish(format_args!("({}) {}", record.level(), message)))
        .chain(Box::new(JsLog) as Box<dyn log::Log>)
        .apply()
        .expect("expected setup_logging to only ever be called once per instance");

    // forward panics to console
    std::panic::set_hook(Box::new(|info| {
        console::log_1(&JsString::from(format!("{info}")))
    }));
}
