//! Sequential cache-aside demonstration with a synthetic public catalog.
//! The in-memory origin is a test fixture, not a durable database or server.
use redis::{Client, Connection, RedisResult};
use std::{
    env,
    io::{self, Write},
    process::ExitCode,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const TIMEOUT: Duration = Duration::from_millis(250);
const TTL_MS: u64 = 1_000;

struct Origin {
    value: String,
    reads: usize,
}

impl Origin {
    fn read(&mut self) -> String {
        // Artificial cost makes the example observable, not a benchmark.
        thread::sleep(Duration::from_millis(25));
        self.reads += 1;
        self.value.clone()
    }
}

#[derive(Debug, PartialEq)]
enum From {
    Cache,
    Origin,
}

struct Cache {
    connection: Option<Connection>,
    key: String,
    errors: usize,
}

fn cache_key(namespace: &str) -> Result<String, &'static str> {
    if namespace.is_empty()
        || namespace.len() > 64
        || !namespace
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("namespace must contain 1-64 ASCII letters, digits, or hyphens");
    }
    Ok(format!(
        "oracle:cache-demo:{namespace}:v1:public-product:42"
    ))
}

impl Cache {
    fn connect(url: &str, namespace: &str) -> Result<Self, &'static str> {
        let key = cache_key(namespace)?;
        // Do not put URLs or credentials into errors/logs.
        let client = Client::open(url).map_err(|_| "invalid Redis connection configuration")?;
        let connection = client.get_connection_with_timeout(TIMEOUT).and_then(|c| {
            c.set_read_timeout(Some(TIMEOUT))?;
            c.set_write_timeout(Some(TIMEOUT))?;
            Ok(c)
        });
        let errors = usize::from(connection.is_err());
        Ok(Self {
            connection: connection.ok(),
            key,
            errors,
        })
    }

    fn disable(&mut self) {
        // A short-lived CLI stops trying after one failure. A service needs a
        // bounded reconnect/backoff policy and a separate origin concurrency limit.
        self.connection = None;
        self.errors += 1;
    }

    fn read(&mut self, origin: &mut Origin) -> (String, From) {
        if let Some(connection) = &mut self.connection {
            let result: RedisResult<Option<String>> =
                redis::cmd("GET").arg(&self.key).query(connection);
            match result {
                Ok(Some(value)) => return (value, From::Cache),
                Ok(None) => {}
                Err(_) => self.disable(),
            }
        }
        let value = origin.read();
        if let Some(connection) = &mut self.connection {
            // One command applies value and TTL together, avoiding immortal keys
            // if a separate EXPIRE command were lost. No negative caching here.
            let result: RedisResult<()> = redis::cmd("SET")
                .arg(&self.key)
                .arg(&value)
                .arg("PX")
                .arg(TTL_MS)
                .query(connection);
            if result.is_err() {
                self.disable();
            }
        }
        (value, From::Origin)
    }

    fn invalidate(&mut self) {
        if let Some(connection) = &mut self.connection {
            let result: RedisResult<usize> = redis::cmd("DEL").arg(&self.key).query(connection);
            if result.is_err() {
                self.disable();
            }
        }
    }
}

fn expect_read(cache: &mut Cache, origin: &mut Origin, value: &str, from: From) -> u128 {
    let start = Instant::now();
    assert_eq!(cache.read(origin), (value.to_owned(), from));
    start.elapsed().as_micros()
}

fn scenario(mode: &str, cache: &mut Cache) -> Result<(), &'static str> {
    let mut origin = Origin {
        value: "edition-1".into(),
        reads: 0,
    };
    if mode == "fallback" {
        if cache.connection.is_some() {
            return Err("fallback check requires an unavailable Redis endpoint");
        }
        expect_read(cache, &mut origin, "edition-1", From::Origin);
        origin.value = "edition-2".into();
        cache.invalidate();
        expect_read(cache, &mut origin, "edition-2", From::Origin);
        assert_eq!(origin.reads, 2);
        assert_eq!(cache.errors, 1);
        println!("PASS unavailable: origin_reads=2 cache_errors=1");
        return Ok(());
    }
    if cache.connection.is_none() {
        return Err("this check requires a reachable isolated Redis instance");
    }
    let cold_us = expect_read(cache, &mut origin, "edition-1", From::Origin);
    let warm_us = expect_read(cache, &mut origin, "edition-1", From::Cache);
    assert_eq!(origin.reads, 1);

    if mode == "outage" {
        // The verifier stops only its own Redis process/container at this point.
        println!("READY_FOR_OUTAGE");
        io::stdout().flush().map_err(|_| "cannot flush output")?;
        let mut input = String::new();
        if io::stdin()
            .read_line(&mut input)
            .map_err(|_| "cannot read outage signal")?
            == 0
        {
            return Err("missing outage signal");
        }
        // Model a committed origin write followed by failed invalidation.
        origin.value = "edition-2".into();
        cache.invalidate();
        if cache.connection.is_some() {
            return Err("Redis must be stopped for the outage check");
        }
        expect_read(cache, &mut origin, "edition-2", From::Origin);
        expect_read(cache, &mut origin, "edition-2", From::Origin);
        assert_eq!(origin.reads, 3);
        assert_eq!(cache.errors, 1);
        println!("PASS outage: committed_origin_retained=true origin_reads=3 cache_errors=1");
        return Ok(());
    }

    origin.value = "edition-2".into();
    cache.invalidate(); // In an application, only after the origin transaction commits.
    expect_read(cache, &mut origin, "edition-2", From::Origin);
    expect_read(cache, &mut origin, "edition-2", From::Cache);
    thread::sleep(Duration::from_millis(TTL_MS + 50));
    expect_read(cache, &mut origin, "edition-2", From::Origin);
    assert_eq!(origin.reads, 3);
    assert_eq!(cache.errors, 0);
    // Timings are observations; the assertions prove behavior and origin-read counts.
    println!(
        "PASS cache: miss hit invalidation expiry; origin_reads=3 cold_us={cold_us} warm_us={warm_us} synthetic_delay_ms=25"
    );
    Ok(())
}

fn run() -> Result<(), &'static str> {
    let mode = env::args().nth(1).unwrap_or_else(|| "verify".into());
    if !matches!(mode.as_str(), "verify" | "fallback" | "outage") {
        return Err("usage: oracle-cache-example [verify|fallback|outage]");
    }
    let url = env::var("REDIS_URL").map_err(|_| "set REDIS_URL to a disposable Redis instance")?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "clock precedes the Unix epoch")?
        .as_nanos();
    let namespace = format!("{}-{nanos}", std::process::id());
    let mut cache = Cache::connect(&url, &namespace)?;
    let result = scenario(&mode, &mut cache);
    // Only this run's exact key; never FLUSHDB, FLUSHALL, or a prefix scan.
    cache.invalidate();
    result
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespaces_are_distinct_and_cannot_inject_key_segments() {
        assert_ne!(cache_key("a").unwrap(), cache_key("b").unwrap());
        for value in ["", "a:b", "a*", "a/b", "a b", &"x".repeat(65)] {
            assert!(cache_key(value).is_err());
        }
    }

    #[test]
    fn unavailable_cache_reads_current_origin_without_retries() {
        let mut cache = Cache {
            connection: None,
            key: cache_key("test").unwrap(),
            errors: 1,
        };
        scenario("fallback", &mut cache).unwrap();
    }

    #[test]
    fn broken_configuration_is_reported_without_echoing_credentials() {
        let error = Cache::connect("invalid://secret@example.test", "test")
            .err()
            .unwrap();
        assert_eq!(error, "invalid Redis connection configuration");
    }
}
