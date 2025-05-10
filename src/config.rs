use std::ffi::CStr;

use pgrx::GucRegistry;
use pgrx::GucContext;
use pgrx::GucSetting;
use pgrx::GucFlags;

pub static DATA_CENTER_ID: GucSetting<i32> = GucSetting::<i32>::new(1);
pub static WORKER_ID: GucSetting<i32> = GucSetting::<i32>::new(1);
pub static EPOCH: GucSetting<Option<&'static CStr>> =
    GucSetting::<Option<&'static CStr>>::new(Some(c"2021-01-01 00:00:00 UTC"));

pub unsafe fn init() {
    GucRegistry::define_int_guc(
        "pg_snowflake_id.data_center_id",
        "Integer value",
        "Data center ID",
        &DATA_CENTER_ID,
        0,
        31,
        GucContext::Userset,
        GucFlags::default()
    );

    GucRegistry::define_int_guc(
        "pg_snowflake_id.worker_id",
        "Integer value",
        "Worker ID",
        &WORKER_ID,
        0,
        31,
        GucContext::Userset,
        GucFlags::default()
    );

    GucRegistry::define_string_guc(
        "pg_snowflake_id.epoch",
        "String value",
        "Epoch timestamp in a millisecond",
        &EPOCH,
        GucContext::Userset,
        GucFlags::default(),
    );
}

