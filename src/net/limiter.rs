use crate::{
	cnf::{RATE_LIMIT, RATE_LIMIT_BURST},
	err::Error,
};
use once_cell::sync::OnceCell;
use std::{
	collections::HashMap,
	net::Ipv6Addr,
	sync::Mutex,
	time::{Duration, Instant},
};
use surrealdb::dbs::{Auth, Session};

pub static LIM: OnceCell<Limiter> = OnceCell::new();

pub fn init() -> Result<(), Error> {
	let _ = LIM.set(Default::default());
	// All ok
	Ok(())
}

#[derive(Debug, Eq, PartialEq, Hash)]
enum BlockableUnit {
	/// IPv4 address or IPv6 /48 prefixes
	Ip(Box<str>),
	/// Authed access to a namespace
	Namespace(Box<str>),
}

struct Limits {
	/// How long previous request(s) are counted against the client (millis from utime).
	rate_limited: u32,
	/// How many extra requests have been allowed (counted towards a limit)
	burst_used: u16,
	// Concurrent connections (lower bound estimate)
	concurrency: u16,
}

pub struct Limiter {
	inner: Mutex<Inner>,
	dur_per_req: u32,
	prune_interval: u32,
}

impl Default for Limiter {
	fn default() -> Self {
		Self::new(*RATE_LIMIT, *RATE_LIMIT_BURST)
	}
}

struct Inner {
	limits: HashMap<BlockableUnit, Limits>,
	last_prune: Instant,
}

impl Limiter {
	fn new(rate_limit: u64, burst: usize) -> Self {
		let dur_per_req = Duration::from_nanos(1_000_000_000 / rate_limit);
		Self {
			inner: Mutex::new(Inner {
				limits: Default::default(),
				last_prune: Instant::now(),
			}),
			dur_per_req,
			prune_interval: Duration::from_nanos(
				(dur_per_req.as_nanos() as u64).saturating_mul(1 + burst as u64),
			),
		}
	}

	/// Returns whether a new connection by this
	/// session should be blocked
	pub fn should_allow(&self, session: &Session) -> bool {
		self.should_allow_at(session, Instant::now())
	}

	/// Allows mocking the time in a test
	fn should_allow_at(&self, session: &Session, now: Instant) -> bool {
		let blockable_unit = match (&*session.au, session.ip.as_deref()) {
			(Auth::Kv, _) => {
				// If you have the root password, you are never rate-limited
				return true;
			}
			(Auth::Ns(ns) | Auth::Db(ns, _), _) => BlockableUnit::Namespace(Box::from(ns.as_str())),
			(_, Some(ip_port)) => {
				let ip = ip_port.rsplit_once(':').map(|(ip, _port)| ip).unwrap_or(ip_port);
				let ip = if let Ok(ipv6) = ip.parse::<Ipv6Addr>() {
					let mut octets = ipv6.octets();
					// Ignore parts of the address that are easily spoofed
					octets[6..].iter_mut().for_each(|o| *o = 0);
					Ipv6Addr::from(octets).to_string().into_boxed_str()
				} else {
					Box::from(ip)
				};
				BlockableUnit::Ip(ip)
			}
			_ => {
				// It's fine not to have namespace auth but lack of IP means something
				// wrong involving warp
				debug_assert!(false, "no IP in session");
				return true;
			}
		};

		// TODO: asynchronously consult the KVs for heavy-hitters.
		let mut tx = kvs.transaction(false, false).await?;

		let mut inner = self.inner.lock().unwrap();

		let limits = inner.limits.entry(blockable_unit).or_insert(Limits {
			rate_limited_until: now,
			burst_used: 0,
		});

		let ok = if now > limits.rate_limited_until {
			// Limit has fully expired
			limits.burst_used = 0;
			limits.rate_limited_until = now;
			true
		} else if limits.burst_used <= *RATE_LIMIT_BURST {
			// Allowable burst
			limits.burst_used += 1;
			limits.rate_limited_until += self.dur_per_req;
			true
		} else {
			// Excessive burst
			false
		};

		// TODO: Check concurrent connections

		// See if we can prune some elements.
		if (now - inner.last_prune) > self.prune_interval {
			inner.last_prune = now;
			inner.limits.retain(|_, l| l.rate_limited_until > now);
		}

		ok
	}
}

/// Implements a space efficient timestamp with millisecond precision
pub mod utime {
	use std::sync::atomic::{AtomicU64, Ordering};

	/// If a timestamp is persisted (e.g. in the KVS) for longer than this,
	/// it may exhibit unpredictable behavior.
	pub const EXPIRY: u32 = u32::MAX / 4;

	/// Returns an opaque timesamp with millisecond precision.
	///
	/// Logically although not numerically monotonic.
	pub fn now() -> u32 {
		use std::time::SystemTime;

		let unix_millis = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		static LAST: AtomicU64 = AtomicU64::new(0);

		let monotonic_millis = LAST
			.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |last| Some(last.max(unix_millis)))
			.unwrap()
			.max(unix_millis);

		let since_epoch = monotonic_millis as u32;
		since_epoch
	}

	/// Returns if `after` is equal to or later than `before`.
	pub fn is_on_or_after(after: u32, before: u32) -> bool {
		// On or after (no overflow case):
		// |--------------------------->|
		//          b    a
		//       `difference` is positive with magnitude close to 0
		//
		// On or after (overflow case)
		// |---------------------------->|
		// |  a                       b  |
		//       `difference is negative with a magnitude close to u32::MAX
		//
		// Before (no overflow case):
		// |--------------------------->|
		//          a    b
		//       `difference` is negative with magnitude close to 0
		//
		// Before (overflow case)
		// |---------------------------->|
		// |  b                       a  |
		//       `difference is positive with a magnitude close to u32::MAX
		let difference = after as i64 - before as i64;
		(0..=difference).contains(&EXPIRY) || (u32::MAX as i64..u32::MAX + EXPIRY)
	}

	pub fn add(a: u32, b: u32) -> u32 {
		a.wrapping_add(b)
	}
}

#[cfg(test)]
mod tests {
	use super::Limiter;
	use rand::{thread_rng, Rng};
	use std::{
		net::Ipv4Addr,
		time::{Duration, Instant},
	};
	use surrealdb::dbs::Session;

	const RATE_LIMIT: u64 = 3;
	const RATE_LIMIT_BURST: usize = 5;

	fn limiter() -> Limiter {
		Limiter::new(RATE_LIMIT, RATE_LIMIT_BURST)
	}

	#[test]
	fn rate() {
		let session = Session {
			ip: Some("0.0.0.0".to_owned()),
			..Default::default()
		};

		// Returns true iff requests at this rate are all allowed
		let is_allowed = |rate: f64| {
			let limiter = limiter();
			let mut now = Instant::now();

			for _ in 0..RATE_LIMIT_BURST * 1000 {
				if !limiter.should_allow_at(&session, now) {
					return false;
				}
				now += Duration::from_nanos(1 + (1_000_000_000 as f64 / rate) as u64)
			}
			true
		};

		for ten_times_rate in
			(RATE_LIMIT.saturating_sub(10) + 1) * 10..=RATE_LIMIT.saturating_add(10) * 10
		{
			let rate = ten_times_rate as f64 * 0.1;
			assert_eq!(is_allowed(rate), rate <= RATE_LIMIT as f64, "rate: {:.1}", rate);
		}
	}

	#[test]
	fn burst() {
		let limiter = limiter();
		let mut now = Instant::now();

		let session = Session {
			ip: Some("0.0.0.0".to_owned()),
			..Default::default()
		};

		for i in 0..RATE_LIMIT_BURST * 1000 {
			// Essentially zero time has passed
			assert_eq!(limiter.should_allow_at(&session, now), i <= RATE_LIMIT_BURST, "{i}");
			now += Duration::from_nanos(1);
		}
	}

	#[test]
	fn expiry() {
		let limiter = limiter();

		let mut rng = thread_rng();
		let mut now = Instant::now();

		for _ in 0..1000 {
			let session = Session {
				ip: Some(Ipv4Addr::new(rng.gen(), rng.gen(), rng.gen(), rng.gen()).to_string()),
				..Default::default()
			};

			assert!(limiter.should_allow_at(&session, now));

			now += Duration::from_secs(1);

			let len = limiter.inner.lock().unwrap().limits.len();
			assert!(len < 100, "{}", len);
		}
	}
}
