//! 配置日志追踪

use time::format_description::FormatItem;
use time::UtcOffset;
use tracing::{debug, error, info, Level, trace, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const LOG_DIR: &str = "./logs";
const F_PFX_NOR: &str = "bridge_log.log";
const F_PFX_ERR: &str = "bridge_err.log";

/// 配置时区和时间格式
fn get_timer(t_fmt: Vec<FormatItem>) -> OffsetTime<Vec<FormatItem>> {
    match UtcOffset::from_hms(8, 0, 0) {
        Ok(ofs) => OffsetTime::new(ofs, t_fmt),
        Err(e) => {
            eprintln!("配置时区异常！{:#?}", e);
            panic!("配置时区异常！");
        }
    }
}

/// 初始化日志
pub fn init_logger() -> (WorkerGuard, WorkerGuard) {
    println!("init logger...");
    let t_fmt1 = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]").unwrap();
    let t_fmt2 = time::format_description::parse("[hour]:[minute]:[second].[subsecond digits:3]").unwrap();

    // 日志文件
    let (ff, nl_guard) = tracing_appender::non_blocking(rolling::never(LOG_DIR, F_PFX_NOR));
    let f_normal = fmt::layer()
        .with_ansi(false)
        .with_writer(
            ff.with_min_level(Level::WARN)
                .with_max_level(Level::INFO));
    let (ff, el_guard) = tracing_appender::non_blocking(rolling::never(LOG_DIR, F_PFX_ERR));
    let f_error = fmt::layer()
        .with_ansi(false)
        .with_writer(
            ff.with_max_level(Level::ERROR)
                .with_max_level(Level::ERROR));
    let (f_normal, f_error) = {
        let timer = get_timer(t_fmt1);
        (
            f_normal.with_timer(timer.clone()),
            f_error.with_timer(timer),
        )
    };

    // 标准输出
    let timer = get_timer(t_fmt2);
    let std_out = fmt::layer()
        .with_timer(timer)
        .with_ansi(true)
        .with_writer(std::io::stdout);

    // 注册
    tracing_subscriber::registry()
        .with(EnvFilter::from_env("MSG_BRIDGE"))
        .with(std_out)
        .with(f_normal)
        .with(f_error)
        .init();

    // color_eyre 处理 panic
    if let Err(e) = color_eyre::install() {
        error!("color_eyre 配置异常！{:#?}", e);
    }

    trace!("logger ready.");
    debug!("logger ready.");
    info!("logger ready.");
    warn!("logger ready.");
    error!("logger ready.");
    debug!("env: {:#?}", EnvFilter::from_default_env());
    (nl_guard, el_guard)
}
