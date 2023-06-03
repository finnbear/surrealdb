use once_cell::sync::OnceCell;
use std::{net::SocketAddr, path::PathBuf, time::Duration};

pub static CF: OnceCell<Config> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct Config {
	pub strict: bool,
	pub query_timeout: Option<Duration>,
	pub bind: SocketAddr,
	pub path: String,
	pub user: String,
	pub pass: Option<String>,
	pub crt: Option<PathBuf>,
	pub key: Option<PathBuf>,
}
