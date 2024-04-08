use std::io::Write;

pub mod command;
pub mod message;
pub mod acl;
pub mod connection;

pub fn init_env_logger() {
    env_logger::Builder::from_env(env_logger::Env::default())
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                record.target(),
                record.level(),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Trace)
        .write_style(env_logger::WriteStyle::Always)
        .init();
}

#[cfg(test)]
mod tests {}
