//! # upid_pg
//!
//! `upid_pg` is a thin wrapper for [upid](https://crates.io/crates/upid)
//! providing the UPID datatype and generator as a Postgres extension
//!
//! The code below is based largely on the following:
//! https://github.com/pksunkara/pgx_ulid

use core::ffi::CStr;
use inner_upid::Upid as InnerUpid;
use pgrx::{
    pg_shmem_init,
    pg_sys::{Datum, Oid},
    prelude::*,
    rust_regtypein,
    shmem::*,
    PgLwLock, StringInfo, Uuid,
};

pgrx::pg_module_magic!();

static SHARED_UPID: PgLwLock<u128> = PgLwLock::new();

#[pg_guard]
pub extern "C" fn _PG_init() {
    pg_shmem_init!(SHARED_UPID);
}

#[allow(non_camel_case_types)]
#[derive(
    PostgresType, PostgresEq, PostgresHash, PostgresOrd, Debug, PartialEq, PartialOrd, Eq, Hash, Ord,
)]
#[inoutfuncs]
pub struct upid(u128);

impl InOutFuncs for upid {
    #[inline]
    fn input(input: &CStr) -> Self
    where
        Self: Sized,
    {
        let val = input.to_str().unwrap();
        let inner = InnerUpid::from_string(val)
            .unwrap_or_else(|err| panic!("invalid input syntax for type upid: \"{val}\": {err}"));

        upid(inner.0)
    }

    #[inline]
    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&InnerUpid(self.0).to_string())
    }
}

impl IntoDatum for upid {
    #[inline]
    fn into_datum(self) -> Option<Datum> {
        self.0.to_ne_bytes().into_datum()
    }

    #[inline]
    fn type_oid() -> Oid {
        rust_regtypein::<Self>()
    }
}

impl FromDatum for upid {
    #[inline]
    unsafe fn from_polymorphic_datum(datum: Datum, is_null: bool, typoid: Oid) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8] = FromDatum::from_polymorphic_datum(datum, is_null, typoid)?;

        let mut len_bytes = [0u8; 16];
        len_bytes.copy_from_slice(bytes);

        Some(upid(u128::from_ne_bytes(len_bytes)))
    }
}

#[pg_extern]
fn gen_upid(prefix: &str) -> upid {
    upid(InnerUpid::new(prefix).unwrap().0)
}

#[pg_extern(immutable, parallel_safe)]
fn upid_from_uuid(input: Uuid) -> upid {
    let mut bytes = *input.as_bytes();
    bytes.reverse();
    upid(u128::from_ne_bytes(bytes))
}

#[pg_extern(immutable, parallel_safe)]
fn upid_to_uuid(input: upid) -> Uuid {
    let mut bytes = input.0.to_ne_bytes();
    bytes.reverse();
    Uuid::from_bytes(bytes)
}

#[pg_extern(immutable, parallel_safe)]
fn upid_to_bytea(input: upid) -> Vec<u8> {
    let mut bytes = input.0.to_ne_bytes();
    bytes.reverse();
    bytes.to_vec()
}

#[pg_extern(immutable, parallel_safe)]
fn upid_to_timestamp(input: upid) -> Timestamp {
    let inner_seconds = (InnerUpid(input.0).milliseconds() as f64) / 1000.0;
    to_timestamp(inner_seconds).into()
}

extension_sql!(
    r#"
CREATE CAST (uuid AS upid) WITH FUNCTION upid_from_uuid(uuid) AS IMPLICIT;
CREATE CAST (upid AS uuid) WITH FUNCTION upid_to_uuid(upid) AS IMPLICIT;
CREATE CAST (upid AS bytea) WITH FUNCTION upid_to_bytea(upid) AS IMPLICIT;
CREATE CAST (upid AS timestamp) WITH FUNCTION upid_to_timestamp(upid) AS IMPLICIT;
"#,
    name = "upid_casts"
);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    const INT: u128 = 2080078208899192275105038102332577142;
    const TEXT: &str = "user_2acdrlkjmhs6ar53taem6a";
    const UUID: &str = "01909bc6-0f93-7043-5c61-c99524d61576";
    const BYTEA: &[u8] = &[
        1, 144, 155, 198, 15, 147, 112, 67, 92, 97, 201, 149, 36, 214, 21, 118,
    ];
    const TIMESTAMP: &str = "2024-07-10 08:32:46.848";

    #[pg_test]
    fn test_null_to_upid() {
        let result = Spi::get_one::<upid>("SELECT NULL::upid;").unwrap();
        assert_eq!(None, result);
    }

    #[pg_test]
    fn test_string_to_upid() {
        let result = Spi::get_one::<upid>(&format!("SELECT '{TEXT}'::upid;")).unwrap();
        assert_eq!(Some(upid(INT)), result);
    }

    #[pg_test]
    fn test_upid_to_string() {
        let result = Spi::get_one::<&str>(&format!("SELECT '{TEXT}'::upid::text;")).unwrap();
        assert_eq!(Some(TEXT), result);
    }

    #[pg_test]
    fn test_string_to_upid_lowercase() {
        let result = Spi::get_one::<upid>(&format!("SELECT LOWER('{TEXT}')::upid;")).unwrap();
        assert_eq!(Some(upid(INT)), result);
    }

    #[pg_test]
    #[should_panic = "invalid input syntax for type upid: \"01GV5PA9EQG7D82Q3Y4PKBZSY\": invalid length"]
    fn test_string_to_upid_invalid_length() {
        let _ = Spi::get_one::<upid>("SELECT '01GV5PA9EQG7D82Q3Y4PKBZSY'::upid;");
    }

    #[pg_test]
    #[should_panic = "invalid input syntax for type upid: \"01GV5PA9EQG7D82Q3Y4PKBZSYU\": invalid character"]
    fn test_string_to_upid_invalid_char() {
        let _ = Spi::get_one::<upid>("SELECT '01GV5PA9EQG7D82Q3Y4PKBZSYU'::upid;");
    }

    #[pg_test]
    fn test_upid_to_timestamp() {
        let result = Spi::get_one::<&str>(&format!(
            "SET TIMEZONE TO 'UTC'; SELECT '{TEXT}'::upid::timestamp::text;"
        ))
        .unwrap();
        assert_eq!(Some(TIMESTAMP), result);
    }

    #[pg_test]
    fn test_upid_to_uuid() {
        let result = Spi::get_one::<&str>(&format!("SELECT '{TEXT}'::upid::uuid::text;")).unwrap();
        assert_eq!(Some(UUID), result);
    }

    #[pg_test]
    fn test_upid_to_bytea() {
        let result = Spi::get_one::<&[u8]>(&format!("SELECT '{TEXT}'::upid::bytea;")).unwrap();

        assert_eq!(Some(BYTEA), result);
    }

    #[pg_test]
    fn test_uuid_to_upid() {
        let result = Spi::get_one::<upid>(&format!("SELECT '{UUID}'::uuid::upid;")).unwrap();
        assert_eq!(Some(upid(INT)), result);
    }

    #[pg_test]
    fn test_generate() {
        let result = Spi::get_one::<upid>("SELECT gen_upid('user');").unwrap();
        assert!(result.is_some());
    }

    #[pg_test]
    fn test_hash() {
        Spi::run(
            "CREATE TABLE foo (
                id upid,
                data TEXT
            );

            CREATE TABLE bar (
                id upid,
                foo_id upid
            );

            INSERT INTO foo DEFAULT VALUES;
            INSERT INTO bar DEFAULT VALUES;

            SELECT *
            FROM bar
            JOIN foo ON bar.id = foo.id;",
        )
        .unwrap();
    }

    #[pg_test]
    fn test_commutator() {
        Spi::run(
            "CREATE TABLE foo (
                id upid,
                data TEXT
            );

            CREATE TABLE bar (
                id upid
            );

            SELECT *
            FROM bar
            JOIN foo ON bar.id = foo.id;",
        )
        .unwrap();
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
