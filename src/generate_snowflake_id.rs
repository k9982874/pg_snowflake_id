use pgrx::pg_extern;

use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::common::{OrPgrxError};

use crate::config;

use crate::snowflake::Snowflake;
use crate::snowflake::{ set_epoch };

lazy_static! {
    static ref GLOBAL_GENERATOR: Mutex<Option<Snowflake>> = Mutex::new(None);
}

#[pg_extern]
fn generate_snowflake_id() -> i64 {
    let mut generator = GLOBAL_GENERATOR.lock().unwrap();
    if generator.is_none() {
        let center_id = config::DATA_CENTER_ID.get() as u64;
        let worker_id = config::WORKER_ID.get() as u64;
        let epoch = config::EPOCH.get()
            .or_pgrx_error("failed to read EPOCH value")
            .to_str()
            .or_pgrx_error("failed to convert EPOCH to string");

        set_epoch(epoch).or_pgrx_error("failed to set EPOCH");

        *generator = Some(
            Snowflake::new(center_id, worker_id)
                .or_pgrx_error("failed to create snowflake generator"),
        );
    }

    generator
        .as_mut()
        .or_pgrx_error("snowflake generator not initialized")
        .next_id()
        .or_pgrx_error("failed to generate snowflake id") as i64
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_generate_snowflake_id() {
        let id = crate::generate_snowflake_id();
        assert!(id > 0);
    }
}

