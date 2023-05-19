pub mod base;
pub mod clear;
pub mod parse;
pub mod signin;
pub mod signup;
pub mod token;
pub mod verify;

use crate::cli::Config;
use crate::err::Error;

pub const BASIC: &str = "Basic ";
pub const TOKEN: &str = "Bearer ";

const LOG: &str = "surrealdb::iam";

pub async fn init(opt: &Config) -> Result<(), Error> {
	// Log authentication options
	match opt.pass {
		Some(_) => {
			info!(target: LOG, "Root authentication is enabled");
			info!(target: LOG, "Root username is '{}'", opt.user);
		}
		None => info!(target: LOG, "Root authentication is disabled"),
	};
	// All ok
	Ok(())
}
