//! This binary is the web-platform server for [SurrealDB](https://surrealdb.com) the
//! ultimate cloud database for tomorrow's applications. SurrealDB is a scalable,
//! distributed, collaborative, document-graph database for the realtime web.
//!
//! This binary can be used to start a database server instance using an embedded
//! in-memory datastore, or an embedded datastore persisted to disk. In addition, it
//! can be used in distributed mode by connecting to a distributed [TiKV](https://tikv.org)
//! key-value store.

#![deny(clippy::mem_forget)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate log;

#[macro_use]
mod mac;

mod cli;
mod cnf;
mod dbs;
mod env;
mod err;
mod iam;
mod net;
mod o11y;
mod rpc;

use cli::Cli;
use std::future::Future;
use std::process::ExitCode;
use clap::Parser;

fn main() -> ExitCode {
	// Initiate the command line
	with_enough_stack(cli::init(Cli::parse()))
}

/// Rust's default thread stack size of 2MiB doesn't allow sufficient recursion depth.
fn with_enough_stack<T>(fut: impl Future<Output = T> + Send) -> T {
	let stack_size = 8 * 1024 * 1024;

	// Stack frames are generally larger in debug mode.
	#[cfg(debug_assertions)]
	let stack_size = stack_size * 2;

	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.thread_stack_size(stack_size)
		.build()
		.unwrap()
		.block_on(fut)
}

#[cfg(test)]
mod test {
	use std::sync::{Arc, Mutex};

	fn run(args: &str) -> String {
		#[derive(Clone)]
		struct Mock(Arc<Mutex<Vec<u8>>>);
		impl std::io::Write for Mock {
			fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
				self.0.lock().unwrap().write(buf)
			}

			fn flush(&mut self) -> std::io::Result<()> {
				// No-op.
				Ok(())
			}
		}

		let mock = Mock(Arc::new(Mutex::new(Vec::new())));

		let collector = {
			let mock = mock.clone();
			tracing_subscriber::fmt().with_writer(move || mock.clone()).finish()
		};

		let _exit_code = tracing::subscriber::with_default(collector, || {
			let args = std::iter::once("surreal").chain(args.split_ascii_whitespace());
			use clap::Parser;
			let args = crate::Cli::parse_from(args);
			futures::executor::block_on(crate::cli::init(
				args,
			))
			
		});

		let lock = mock.0.lock().unwrap();
		String::from_utf8(lock.clone()).unwrap()
	}

	#[test]
	fn version() {
		assert_eq!(run("version"), crate::env::release())
	}
}
