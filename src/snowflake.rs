use std::error::Error;
use std::fmt;
use std::sync::{Mutex, PoisonError};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime};

static EPOCH: AtomicU64 = AtomicU64::new(1609459200000); // RFC3339 2021-01-01T00:00:00Z

const TIMESTAMP_BITS: u8 = 41;
const DATACENTER_ID_BITS: u8 = 5;
const WORKER_ID_BITS: u8 = 5;
const SEQUENCE_BITS: u8 = 12;

const MAX_DATACENTER_ID: u64 = (1 << DATACENTER_ID_BITS) - 1;
const MAX_WORKER_ID: u64 = (1 << WORKER_ID_BITS) - 1;
const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;

pub fn set_epoch(rfc3339_str: &str) -> Result<(), chrono::ParseError> {
    let dt = DateTime::parse_from_rfc3339(rfc3339_str)?;
    let millis = dt.timestamp_millis();
    EPOCH.store(millis as u64, Ordering::Relaxed);
    Ok(())
}

#[derive(Debug)]
pub struct Snowflake {
    datacenter_id: u64,
    worker_id: u64,
    state: Mutex<SnowflakeState>,
}

#[derive(Debug)]
struct SnowflakeState {
    last_timestamp: u64,
    sequence: u64,
}

#[derive(Debug)]
pub enum SnowflakeError {
    ClockBackwards,
    InvalidArgument(String),
    PoisonedLock,
}

impl<T> From<PoisonError<T>> for SnowflakeError {
    fn from(_: PoisonError<T>) -> Self {
        SnowflakeError::PoisonedLock
    }
}

impl fmt::Display for SnowflakeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
          SnowflakeError::ClockBackwards => 
              write!(f, "Clock moved backwards, time drift detected"),
          SnowflakeError::InvalidArgument(msg) => 
              write!(f, "Invalid argument: {}", msg),
          SnowflakeError::PoisonedLock => 
              write!(f, "Mutex lock poisoned due to thread panic"),
      }
  }
}

impl Error for SnowflakeError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
      None
  }
}

impl Snowflake {
    pub fn new(datacenter_id: u64, worker_id: u64) -> Result<Self, SnowflakeError> {
        if datacenter_id > MAX_DATACENTER_ID {
            return Err(SnowflakeError::InvalidArgument(format!(
                "Datacenter ID must be between 0 and {}",
                MAX_DATACENTER_ID
            )));
        }
        if worker_id > MAX_WORKER_ID {
            return Err(SnowflakeError::InvalidArgument(format!(
                "Worker ID must be between 0 and {}",
                MAX_WORKER_ID
            )));
        }

        Ok(Self {
            datacenter_id,
            worker_id,
            state: Mutex::new(SnowflakeState {
                last_timestamp: 0,
                sequence: 0,
            }),
        })
    }

    pub fn next_id(&self) -> Result<u64, SnowflakeError> {
        let mut state = self.state.lock()?;
        let mut timestamp = current_timestamp()?;

        if timestamp < state.last_timestamp {
            return Err(SnowflakeError::ClockBackwards);
        }

        if timestamp == state.last_timestamp {
            state.sequence = (state.sequence + 1) & MAX_SEQUENCE;
            if state.sequence == 0 {
                timestamp = wait_next_millis(timestamp)?;
            }
        } else {
            state.sequence = 0;
        }

        state.last_timestamp = timestamp;

        let epoch = EPOCH.load(Ordering::Relaxed);

        let timestamp_part = (timestamp - epoch) << (DATACENTER_ID_BITS + WORKER_ID_BITS + SEQUENCE_BITS);
        let datacenter_part = self.datacenter_id << (WORKER_ID_BITS + SEQUENCE_BITS);
        let worker_part = self.worker_id << SEQUENCE_BITS;
        let sequence_part = state.sequence;

        Ok(timestamp_part | datacenter_part | worker_part | sequence_part)
    }
}

fn current_timestamp() -> Result<u64, SnowflakeError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SnowflakeError::ClockBackwards)
        .map(|d| d.as_millis() as u64)
}

fn wait_next_millis(last_timestamp: u64) -> Result<u64, SnowflakeError> {
    let mut timestamp = current_timestamp()?;
    while timestamp <= last_timestamp {
        std::thread::yield_now();
        timestamp = current_timestamp()?;
    }
    Ok(timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let clock_err = SnowflakeError::ClockBackwards;
        assert_eq!(
            format!("{}", clock_err),
            "Clock moved backwards, time drift detected"
        );

        let arg_err = SnowflakeError::InvalidArgument("test message".into());
        assert_eq!(
            format!("{}", arg_err),
            "Invalid argument: test message"
        );

        let lock_err = SnowflakeError::PoisonedLock;
        assert_eq!(
            format!("{}", lock_err),
            "Mutex lock poisoned due to thread panic"
        );
    }

    #[test]
    fn test_error_source() {
        let err = SnowflakeError::ClockBackwards;
        assert!(err.source().is_none());
    }

    #[test]
    fn test_invalid_arguments() {
        match Snowflake::new(32, 0) {
            Err(SnowflakeError::InvalidArgument(msg)) => 
                assert!(msg.contains("must be between 0 and 31")),
            _ => panic!("Should return InvalidArgument error"),
        }

        match Snowflake::new(0, 32) {
            Err(SnowflakeError::InvalidArgument(msg)) => 
                assert!(msg.contains("must be between 0 and 31")),
            _ => panic!("Should return InvalidArgument error"),
        }
    }
}

