use crate::ctx::Context;
use crate::err::Error;
use crate::fnc::args::shim;
use crate::sql::value::Value;

pub mod args;
pub mod array;
pub mod cast;
pub mod count;
pub mod crypto;
pub mod future;
pub mod geo;
pub mod http;
pub mod is;
pub mod math;
pub mod operate;
pub mod parse;
pub mod rand;
pub mod script;
pub mod session;
pub mod string;
pub mod time;
pub mod r#type;
pub mod util;

// Attempts to run any function
pub async fn run(ctx: &Context<'_>, name: &str, args: Vec<Value>) -> Result<Value, Error> {
	macro_rules! dispatch {
		($name: ident, $ctx: expr, $args: ident, $($function_name: literal => $function: path),+, $((ctx) $ctx_function_name: literal => $ctx_function: path),+, $($async_function_name: literal => async $async_function: path),+) => {
			{
				match $name {
					$($function_name => $function(shim($name, $args)?),)+
					$($ctx_function_name => $ctx_function($ctx, shim($name, $args)?),)+
					$($async_function_name => $async_function(shim($name, $args)?).await,)+
					_ => unreachable!()
				}
			}
		}
	}

	dispatch!(
		name,
		ctx,
		args,
		"array::combine" => array::combine,
		"array::concat" => array::concat,
		"array::difference" => array::difference,
		"array::distinct" => array::distinct,
		"array::intersect" => array::intersect,
		"array::len" => array::len,
		"array::sort" => array::sort,
		"array::union" => array::union,
		"array::sort::asc" => array::sort::asc,
		"array::sort::desc" => array::sort::desc,
		"count" => count::count,
		"crypto::md5" => crypto::md5,
		"crypto::sha1" => crypto::sha1,
		"crypto::sha256" => crypto::sha256,
		"crypto::sha512" => crypto::sha512,
		"crypto::argon2::compare" => crypto::argon2::cmp,
		"crypto::argon2::generate" => crypto::argon2::gen,
		"crypto::pbkdf2::compare" => crypto::pbkdf2::cmp,
		"crypto::pbkdf2::generate" => crypto::pbkdf2::gen,
		"crypto::scrypt::compare" => crypto::scrypt::cmp,
		"crypto::scrypt::generate" => crypto::scrypt::gen,
		"geo::area" => geo::area,
		"geo::bearing" => geo::bearing,
		"geo::centroid" => geo::centroid,
		"geo::distance" => geo::distance,
		"geo::hash::decode" => geo::hash::decode,
		"geo::hash::encode" => geo::hash::encode,
		"is::alphanum" => is::alphanum,
		"is::alpha" => is::alpha,
		"is::ascii" => is::ascii,
		"is::domain" => is::domain,
		"is::email" => is::email,
		"is::hexadecimal" => is::hexadecimal,
		"is::latitude" => is::latitude,
		"is::longitude" => is::longitude,
		"is::numeric" => is::numeric,
		"is::semver" => is::semver,
		"is::uuid" => is::uuid,
		"math::abs" => math::abs,
		"math::bottom" => math::bottom,
		"math::ceil" => math::ceil,
		"math::fixed" => math::fixed,
		"math::floor" => math::floor,
		"math::interquartile" => math::interquartile,
		"math::max" => math::max,
		"math::mean" => math::mean,
		"math::median" => math::median,
		"math::midhinge" => math::midhinge,
		"math::min" => math::min,
		"math::mode" => math::mode,
		"math::nearestrank" => math::nearestrank,
		"math::percentile" => math::percentile,
		"math::product" => math::product,
		"math::roun" => math::round,
		"math::spread" => math::spread,
		"math::sqrt" => math::sqrt,
		"math::stddev" => math::stddev,
		"math::sum" => math::sum,
		"math::top" => math::top,
		"math::trimean" => math::trimean,
		"math::variance" => math::variance,
		"parse::email::host" => parse::email::host,
		"parse::email::user" => parse::email::user,
		"parse::url::domain" => parse::url::domain,
		"parse::url::fragment" => parse::url::fragment,
		"parse::url::host" => parse::url::host,
		"parse::url::path" => parse::url::path,
		"parse::url::port" => parse::url::port,
		"parse::url::query" => parse::url::query,
		"rand::bool" => rand::bool,
		"rand::enum" => rand::r#enum,
		"rand::float" => rand::float,
		"rand::guid" => rand::guid,
		"rand::int" => rand::int,
		"rand::string" => rand::string,
		"rand::time" => rand::time,
		"rand::uuid" => rand::uuid,
		"rand" => rand::rand,
		"string::concat" => string::concat,
		"string::endsWith" => string::ends_with,
		"string::join" => string::join,
		"string::length" => string::length,
		"string::lowercase" => string::lowercase,
		"string::repeat" => string::repeat,
		"string::replace" => string::replace,
		"string::reverse" => string::reverse,
		"string::slice" => string::slice,
		"string::slug" => string::slug,
		"string::split" => string::split,
		"string::startsWith" => string::starts_with,
		"string::trim" => string::trim,
		"string::uppercase" => string::uppercase,
		"string::words" => string::words,
		"time::day" => time::day,
		"time::floor" => time::floor,
		"time::group" => time::group,
		"time::hour" => time::hour,
		"time::mins" => time::mins,
		"time::month" => time::month,
		"time::nano" => time::nano,
		"time::now" => time::now,
		"time::round" => time::round,
		"time::secs" => time::secs,
		"time::unix" => time::unix,
		"time::wday" => time::wday,
		"time::week" => time::week,
		"time::yday" => time::yday,
		"time::year" => time::year,
		"type::bool" => r#type::bool,
		"type::datetime" => r#type::datetime,
		"type::decimal" => r#type::decimal,
		"type::duration" => r#type::duration,
		"type::float" => r#type::float,
		"type::int" => r#type::int,
		"type::number" => r#type::number,
		"type::point" => r#type::point,
		"type::regex" => r#type::regex,
		"type::string" => r#type::string,
		"type::table" => r#type::table,
		"type::thing" => r#type::thing,
		(ctx) "session::db" => session::db,
		(ctx) "session::id" => session::id,
		(ctx) "session::ip" => session::ip,
		(ctx) "session::ns" => session::ns,
		(ctx) "session::origin" => session::origin,
		(ctx) "session::sc" => session::sc,
		(ctx) "session::sd" => session::sd,
		"http::head" => async http::head,
		"http::get" => async http::get,
		"http::put" => async http::put,
		"http::post" => async http::post,
		"http::patch" => async http::patch,
		"http::delete" => async http::delete
	)
}
