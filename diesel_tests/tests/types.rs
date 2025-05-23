// FIXME: Review this module to see if we can do these casts in a more backend agnostic way
#![allow(warnings)]
#[cfg(any(feature = "postgres", feature = "mysql"))]
extern crate bigdecimal;
extern crate chrono;

use crate::schema::*;
use diesel::deserialize::FromSqlRow;
#[cfg(feature = "postgres")]
use diesel::pg::Pg;
use diesel::query_dsl::LoadQuery;
use diesel::sql_types::*;
use diesel::*;

#[cfg(any(feature = "postgres", feature = "mysql"))]
use quickcheck::quickcheck;

table! {
    has_timestamps {
        id -> Integer,
        ts -> Timestamp,
    }
}

table! {
    has_time_types(datetime) {
        datetime -> Timestamp,
        date -> Date,
        time -> Time,
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn errors_during_deserialization_do_not_panic() {
    use self::chrono::NaiveDateTime;
    use self::has_timestamps::dsl::*;
    use diesel::result::Error::DeserializationError;

    let connection = &mut connection();
    diesel::sql_query(
        "CREATE TABLE has_timestamps (
        id SERIAL PRIMARY KEY,
        ts TIMESTAMP NOT NULL
    )",
    )
    .execute(connection)
    .unwrap();
    let valid_pg_date_too_large_for_chrono = "'294276/01/01'";
    diesel::sql_query(format!(
        "INSERT INTO has_timestamps (ts) VALUES ({})",
        valid_pg_date_too_large_for_chrono
    ))
    .execute(connection)
    .unwrap();
    let values = has_timestamps.select(ts).load::<NaiveDateTime>(connection);

    match values {
        Err(DeserializationError(_)) => {}
        v => panic!("Expected a deserialization error, got {:?}", v),
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "sqlite")]
fn errors_during_deserialization_do_not_panic() {
    use self::chrono::NaiveDateTime;
    use self::has_timestamps::dsl::*;
    use diesel::result::Error::DeserializationError;

    let connection = &mut connection();
    diesel::sql_query(
        "CREATE TABLE has_timestamps (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ts VARCHAR NOT NULL
    )",
    )
    .execute(connection)
    .unwrap();

    let valid_sqlite_date_too_large_for_chrono = "'294276-01-01 00:00:00'";
    diesel::sql_query(format!(
        "INSERT INTO has_timestamps (ts) VALUES ({})",
        valid_sqlite_date_too_large_for_chrono
    ))
    .execute(connection)
    .unwrap();
    let values = has_timestamps.select(ts).load::<NaiveDateTime>(connection);

    match values {
        Err(DeserializationError(_)) => {}
        v => panic!("Expected a deserialization error, got {:?}", v),
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "sqlite")]
fn test_chrono_types_sqlite() {
    use self::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use self::has_time_types;

    #[derive(Queryable, Insertable)]
    #[diesel(table_name = has_time_types)]
    struct NewTimeTypes {
        datetime: NaiveDateTime,
        date: NaiveDate,
        time: NaiveTime,
    }

    let connection = &mut connection();
    diesel::sql_query(
        "CREATE TABLE has_time_types (
        datetime DATETIME PRIMARY KEY,
        date DATE,
        time TIME
    )",
    )
    .execute(connection)
    .unwrap();

    let dt = NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11);
    let new_time_types = NewTimeTypes {
        datetime: dt,
        date: dt.date(),
        time: dt.time(),
    };

    insert_into(has_time_types::table)
        .values(&new_time_types)
        .execute(connection)
        .unwrap();

    let result = has_time_types::table
        .first::<NewTimeTypes>(connection)
        .unwrap();
    assert_eq!(result.datetime, dt);
    assert_eq!(result.date, dt.date());
    assert_eq!(result.time, dt.time());
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn boolean_from_sql() {
    assert_eq!(true, query_single_value::<Bool, bool>("'t'::bool"));
    assert_eq!(false, query_single_value::<Bool, bool>("'f'::bool"));
}

#[diesel_test_helper::test]
fn nullable_boolean_from_sql() {
    let connection = &mut connection();
    let one = Some(1).into_sql::<diesel::sql_types::Nullable<Integer>>();
    let query = select(one.eq(None::<i32>));
    assert_eq!(Ok(Option::<bool>::None), query.first(connection));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn boolean_to_sql() {
    assert!(query_to_sql_equality::<Bool, bool>("'t'::bool", true));
    assert!(query_to_sql_equality::<Bool, bool>("'f'::bool", false));
    assert!(!query_to_sql_equality::<Bool, bool>("'t'::bool", false));
    assert!(!query_to_sql_equality::<Bool, bool>("'f'::bool", true));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn i16_from_sql() {
    assert_eq!(0, query_single_value::<SmallInt, i16>("0::int2"));
    assert_eq!(-1, query_single_value::<SmallInt, i16>("-1::int2"));
    assert_eq!(1, query_single_value::<SmallInt, i16>("1::int2"));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn i16_to_sql_smallint() {
    assert!(query_to_sql_equality::<SmallInt, i16>("0::int2", 0));
    assert!(query_to_sql_equality::<SmallInt, i16>("-1::int2", -1));
    assert!(query_to_sql_equality::<SmallInt, i16>("1::int2", 1));
    assert!(!query_to_sql_equality::<SmallInt, i16>("0::int2", 1));
    assert!(!query_to_sql_equality::<SmallInt, i16>("-1::int2", 1));
}

#[diesel_test_helper::test]
fn i32_from_sql() {
    assert_eq!(0, query_single_value::<Integer, i32>("0"));
    assert_eq!(-1, query_single_value::<Integer, i32>("-1"));
    assert_eq!(70_000, query_single_value::<Integer, i32>("70000"));
}

#[diesel_test_helper::test]
fn i32_to_sql_integer() {
    assert!(query_to_sql_equality::<Integer, i32>("0", 0));
    assert!(query_to_sql_equality::<Integer, i32>("-1", -1));
    assert!(query_to_sql_equality::<Integer, i32>("70000", 70_000));
    assert!(!query_to_sql_equality::<Integer, i32>("0", 1));
    assert!(!query_to_sql_equality::<Integer, i32>("70000", 69_999));
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn u8_to_sql_integer() {
    assert!(query_to_sql_equality::<Unsigned<TinyInt>, u8>("255", 255));
    assert!(query_to_sql_equality::<Unsigned<TinyInt>, u8>("0", 0));
    assert!(query_to_sql_equality::<Unsigned<TinyInt>, u8>("1", 1));
    assert!(query_to_sql_equality::<Unsigned<TinyInt>, u8>("123", 123));
    assert!(!query_to_sql_equality::<Unsigned<TinyInt>, u8>("0", 1));
    assert!(!query_to_sql_equality::<Unsigned<TinyInt>, u8>("254", 255));
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn u8_from_sql() {
    assert_eq!(0, query_single_value::<Unsigned<TinyInt>, u8>("0"));
    assert_eq!(255, query_single_value::<Unsigned<TinyInt>, u8>("255"));
    assert_ne!(254, query_single_value::<Unsigned<TinyInt>, u8>("255"));
    assert_eq!(123, query_single_value::<Unsigned<TinyInt>, u8>("123"));
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn u16_to_sql_integer() {
    assert!(query_to_sql_equality::<Unsigned<SmallInt>, u16>(
        "65535", 65535
    ));
    assert!(query_to_sql_equality::<Unsigned<SmallInt>, u16>("0", 0));
    assert!(query_to_sql_equality::<Unsigned<SmallInt>, u16>("1", 1));
    assert!(query_to_sql_equality::<Unsigned<SmallInt>, u16>(
        "7000", 7000
    ));
    assert!(!query_to_sql_equality::<Unsigned<SmallInt>, u16>("0", 1));
    assert!(!query_to_sql_equality::<Unsigned<SmallInt>, u16>(
        "50000", 49999
    ));
    assert!(!query_to_sql_equality::<Unsigned<SmallInt>, u16>(
        "64435", 64434
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn u16_from_sql() {
    assert_eq!(0, query_single_value::<Unsigned<SmallInt>, u16>("0"));
    assert_eq!(
        65535,
        query_single_value::<Unsigned<SmallInt>, u16>("65535")
    );
    assert_ne!(
        65534,
        query_single_value::<Unsigned<SmallInt>, u16>("65535")
    );
    assert_eq!(7000, query_single_value::<Unsigned<SmallInt>, u16>("7000"));
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn u32_to_sql_integer() {
    assert!(query_to_sql_equality::<Unsigned<Integer>, u32>(
        "4294967295",
        4294967295
    ));
    assert!(query_to_sql_equality::<Unsigned<Integer>, u32>("0", 0));
    assert!(query_to_sql_equality::<Unsigned<Integer>, u32>("1", 1));
    assert!(query_to_sql_equality::<Unsigned<Integer>, u32>(
        "70000", 70000
    ));
    assert!(!query_to_sql_equality::<Unsigned<Integer>, u32>("0", 1));
    assert!(!query_to_sql_equality::<Unsigned<Integer>, u32>(
        "70000", 69999
    ));
    assert!(!query_to_sql_equality::<Unsigned<Integer>, u32>(
        "4294967295",
        4294967294
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn u32_from_sql() {
    assert_eq!(0, query_single_value::<Unsigned<Integer>, u32>("0"));
    assert_eq!(
        4294967295,
        query_single_value::<Unsigned<Integer>, u32>("4294967295")
    );
    assert_ne!(
        4294967294,
        query_single_value::<Unsigned<Integer>, u32>("4294967295")
    );
    assert_eq!(70000, query_single_value::<Unsigned<Integer>, u32>("70000"));
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn u64_to_sql_integer() {
    assert!(query_to_sql_equality::<Unsigned<BigInt>, u64>(
        "18446744073709551615",
        18446744073709551615
    ));
    assert!(query_to_sql_equality::<Unsigned<BigInt>, u64>("0", 0));
    assert!(query_to_sql_equality::<Unsigned<BigInt>, u64>("1", 1));
    assert!(query_to_sql_equality::<Unsigned<BigInt>, u64>(
        "700000", 700000
    ));
    assert!(!query_to_sql_equality::<Unsigned<BigInt>, u64>("0", 1));
    assert!(!query_to_sql_equality::<Unsigned<BigInt>, u64>(
        "70000", 69999
    ));
    assert!(!query_to_sql_equality::<Unsigned<BigInt>, u64>(
        "18446744073709551615",
        18446744073709551614
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn u64_from_sql() {
    assert_eq!(0, query_single_value::<Unsigned<BigInt>, u64>("0"));
    assert_eq!(
        18446744073709551615,
        query_single_value::<Unsigned<BigInt>, u64>("18446744073709551615")
    );
    assert_ne!(
        18446744073709551614,
        query_single_value::<Unsigned<BigInt>, u64>("18446744073709551615")
    );
    assert_eq!(
        700000,
        query_single_value::<Unsigned<BigInt>, u64>("700000")
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn i64_from_sql() {
    assert_eq!(0, query_single_value::<BigInt, i64>("0::int8"));
    assert_eq!(-1, query_single_value::<BigInt, i64>("-1::int8"));
    assert_eq!(
        283_745_982_374,
        query_single_value::<BigInt, i64>("283745982374::int8")
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn i64_to_sql_bigint() {
    assert!(query_to_sql_equality::<BigInt, i64>("0::int8", 0));
    assert!(query_to_sql_equality::<BigInt, i64>("-1::int8", -1));
    assert!(query_to_sql_equality::<BigInt, i64>(
        "283745982374::int8",
        283_745_982_374
    ));
    assert!(!query_to_sql_equality::<BigInt, i64>("0::int8", 1));
    assert!(!query_to_sql_equality::<BigInt, i64>(
        "283745982374::int8",
        283_745_982_373
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn mysql_json_from_sql() {
    let query = "'true'";
    let expected_value = serde_json::Value::Bool(true);
    assert_eq!(
        expected_value,
        query_single_value::<Json, serde_json::Value>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn mysql_json_to_sql_json() {
    let expected_value = "'false'";
    let value = serde_json::Value::Bool(false);
    assert!(query_to_sql_equality::<Json, serde_json::Value>(
        expected_value,
        value
    ));
}

use std::{f32, f64};

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
#[allow(clippy::float_cmp)]
fn f32_from_sql() {
    assert_eq!(0.0, query_single_value::<Float, f32>("0.0::real"));
    assert_eq!(0.5, query_single_value::<Float, f32>("0.5::real"));
    let nan = query_single_value::<Float, f32>("'NaN'::real");
    assert!(nan.is_nan());
    assert_eq!(
        f32::INFINITY,
        query_single_value::<Float, f32>("'Infinity'::real")
    );
    assert_eq!(
        -f32::INFINITY,
        query_single_value::<Float, f32>("'-Infinity'::real")
    );
}

#[diesel_test_helper::test]
#[cfg(any(feature = "mysql", feature = "sqlite"))]
#[allow(clippy::float_cmp)]
fn f32_from_sql() {
    assert_eq!(0.0, query_single_value::<Float, f32>("0.0"));
    assert_eq!(0.5, query_single_value::<Float, f32>("0.5"));
    // MySQL has no way to represent NaN or Infinity as a literal
    #[cfg(feature = "sqlite")]
    {
        assert_eq!(f32::INFINITY, query_single_value::<Float, f32>("9e999"));
        assert_eq!(-f32::INFINITY, query_single_value::<Float, f32>("-9e999"));
        // SQLite has no way to represent NaN
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
#[allow(clippy::float_cmp)]
fn f32_to_sql() {
    assert!(query_to_sql_equality::<Float, f32>("0.0::real", 0.0));
    assert!(query_to_sql_equality::<Float, f32>("0.5::real", 0.5));
    assert!(query_to_sql_equality::<Float, f32>("'NaN'::real", f32::NAN));
    assert!(query_to_sql_equality::<Float, f32>(
        "'Infinity'::real",
        f32::INFINITY
    ));
    assert!(query_to_sql_equality::<Float, f32>(
        "'-Infinity'::real",
        -f32::INFINITY
    ));
    assert!(!query_to_sql_equality::<Float, f32>("0.0::real", 0.5));
    assert!(!query_to_sql_equality::<Float, f32>("'NaN'::real", 0.0));
    assert!(!query_to_sql_equality::<Float, f32>(
        "'Infinity'::real",
        -f32::INFINITY
    ));
    assert!(!query_to_sql_equality::<Float, f32>(
        "'-Infinity'::real",
        1.0
    ));
}

#[diesel_test_helper::test]
#[cfg(any(feature = "mysql", feature = "sqlite"))]
fn f32_to_sql() {
    assert!(query_to_sql_equality::<Float, f32>("0.0", 0.0));
    assert!(query_to_sql_equality::<Float, f32>("0.5", 0.5));
    // While MySQL will correctly round trip SELECT ? when the bind param is
    // NaN or Infinity, any attempt to insert those values into a row will
    // result in an error, and we have no way to write SELECT ? = NaN,
    // so those cases are untested.
    #[cfg(feature = "sqlite")]
    {
        assert!(query_to_sql_equality::<Float, f32>("9e999", f32::INFINITY));
        assert!(query_to_sql_equality::<Float, f32>(
            "-9e999",
            -f32::INFINITY
        ));
        // SQLite has no way to represent NaN
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
#[allow(clippy::float_cmp)]
fn f64_from_sql() {
    assert_eq!(
        0.0,
        query_single_value::<Double, f64>("0.0::double precision")
    );
    assert_eq!(
        0.5,
        query_single_value::<Double, f64>("0.5::double precision")
    );
    let nan = query_single_value::<Double, f64>("'NaN'::double precision");
    assert!(nan.is_nan());
    assert_eq!(
        f64::INFINITY,
        query_single_value::<Double, f64>("'Infinity'::double precision")
    );
    assert_eq!(
        -f64::INFINITY,
        query_single_value::<Double, f64>("'-Infinity'::double precision")
    );
}

#[diesel_test_helper::test]
#[cfg(any(feature = "mysql", feature = "sqlite"))]
#[allow(clippy::float_cmp)]
fn f64_from_sql() {
    assert_eq!(0.0, query_single_value::<Double, f64>("0.0"));
    assert_eq!(0.5, query_single_value::<Double, f64>("0.5"));
    // MySQL has no way to represent NaN or Infinity as a literal
    #[cfg(feature = "sqlite")]
    {
        assert_eq!(f64::INFINITY, query_single_value::<Double, f64>("9e999"));
        assert_eq!(-f64::INFINITY, query_single_value::<Double, f64>("-9e999"));
        // SQLite has no way to represent NaN
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn f64_to_sql() {
    assert!(query_to_sql_equality::<Double, f64>(
        "0.0::double precision",
        0.0
    ));
    assert!(query_to_sql_equality::<Double, f64>(
        "0.5::double precision",
        0.5
    ));
    assert!(query_to_sql_equality::<Double, f64>(
        "'NaN'::double precision",
        f64::NAN
    ));
    assert!(query_to_sql_equality::<Double, f64>(
        "'Infinity'::double precision",
        f64::INFINITY
    ));
    assert!(query_to_sql_equality::<Double, f64>(
        "'-Infinity'::double precision",
        -f64::INFINITY
    ));
    assert!(!query_to_sql_equality::<Double, f64>(
        "0.0::double precision",
        0.5
    ));
    assert!(!query_to_sql_equality::<Double, f64>(
        "'NaN'::double precision",
        0.0
    ));
    assert!(!query_to_sql_equality::<Double, f64>(
        "'Infinity'::double precision",
        -f64::INFINITY
    ));
    assert!(!query_to_sql_equality::<Double, f64>(
        "'-Infinity'::double precision",
        1.0
    ));
}

#[diesel_test_helper::test]
#[cfg(any(feature = "mysql", feature = "sqlite"))]
fn f64_to_sql() {
    assert!(query_to_sql_equality::<Double, f64>("0.0", 0.0));
    assert!(query_to_sql_equality::<Double, f64>("0.5", 0.5));
    // While MySQL will correctly round trip SELECT ? when the bind param is
    // NaN or Infinity, any attempt to insert those values into a row will
    // result in an error, and we have no way to write SELECT ? = NaN,
    // so those cases are untested.
    #[cfg(feature = "sqlite")]
    {
        assert!(query_to_sql_equality::<Double, f64>("9e999", f64::INFINITY));
        assert!(query_to_sql_equality::<Double, f64>(
            "-9e999",
            -f64::INFINITY
        ));
        // SQLite has no way to represent NaN
    }
}

#[diesel_test_helper::test]
fn string_from_sql() {
    assert_eq!("hello", &query_single_value::<VarChar, String>("'hello'"));
    assert_eq!("world", &query_single_value::<VarChar, String>("'world'"));
}

#[diesel_test_helper::test]
fn str_to_sql_varchar() {
    assert!(query_to_sql_equality::<VarChar, &str>("'hello'", "hello"));
    assert!(query_to_sql_equality::<VarChar, &str>("'world'", "world"));
    assert!(!query_to_sql_equality::<VarChar, &str>("'hello'", "world"));
}

#[diesel_test_helper::test]
fn string_to_sql_varchar() {
    assert!(query_to_sql_equality::<VarChar, String>(
        "'hello'",
        "hello".to_string()
    ));
    assert!(query_to_sql_equality::<VarChar, String>(
        "'world'",
        "world".to_string()
    ));
    assert!(!query_to_sql_equality::<VarChar, String>(
        "'hello'",
        "world".to_string()
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn binary_from_sql() {
    let invalid_utf8_bytes = vec![0x1Fu8, 0x8Bu8];
    assert_eq!(
        invalid_utf8_bytes,
        query_single_value::<Binary, Vec<u8>>("E'\\\\x1F8B'::bytea")
    );
    assert_eq!(
        Vec::<u8>::new(),
        query_single_value::<Binary, Vec<u8>>("''::bytea")
    );
    assert_eq!(
        vec![0u8],
        query_single_value::<Binary, Vec<u8>>("E'\\\\000'::bytea")
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn bytes_to_sql_binary() {
    let invalid_utf8_bytes = vec![0x1Fu8, 0x8Bu8];
    let invalid_utf8_array = [0x1Fu8, 0x8Bu8];
    assert!(query_to_sql_equality::<Binary, Vec<u8>>(
        "E'\\\\x1F8B'::bytea",
        invalid_utf8_bytes.clone()
    ));
    assert!(query_to_sql_equality::<Binary, &[u8]>(
        "E'\\\\x1F8B'::bytea",
        &invalid_utf8_bytes
    ));
    assert!(query_to_sql_equality::<Binary, &[u8; 2]>(
        "E'\\\\x1F8B'::bytea",
        &invalid_utf8_array
    ));
    assert!(!query_to_sql_equality::<Binary, &[u8]>(
        "''::bytea",
        &invalid_utf8_bytes
    ));
    assert!(query_to_sql_equality::<Binary, Vec<u8>>(
        "''::bytea",
        Vec::<u8>::new()
    ));
    assert!(query_to_sql_equality::<Binary, Vec<u8>>(
        "E'\\\\000'::bytea",
        vec![0u8]
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_specific_option_from_sql() {
    assert_eq!(
        Some(true),
        query_single_value::<Nullable<Bool>, Option<bool>>("'t'::bool")
    );
}

#[diesel_test_helper::test]
fn option_from_sql() {
    assert_eq!(
        None,
        query_single_value::<Nullable<Bool>, Option<bool>>("NULL")
    );
    assert_eq!(
        Some(1),
        query_single_value::<Nullable<Integer>, Option<i32>>("1")
    );
    assert_eq!(
        None,
        query_single_value::<Nullable<Integer>, Option<i32>>("NULL")
    );
    assert_eq!(
        Some("Hello!".to_string()),
        query_single_value::<Nullable<VarChar>, Option<String>>("'Hello!'")
    );
    assert_eq!(
        Some("".to_string()),
        query_single_value::<Nullable<VarChar>, Option<String>>("''")
    );
    assert_eq!(
        None,
        query_single_value::<Nullable<VarChar>, Option<String>>("NULL")
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_specific_option_to_sql() {
    assert!(query_to_sql_equality::<Nullable<Bool>, Option<bool>>(
        "'t'::bool",
        Some(true)
    ));
    assert!(query_to_sql_equality::<Nullable<Bool>, Option<bool>>(
        "'f'::bool",
        Some(false)
    ));
    assert!(query_to_sql_equality::<Nullable<Bool>, Option<bool>>(
        "NULL", None
    ));
    assert!(query_to_sql_equality::<Nullable<Bool>, Option<bool>>(
        "NULL::bool",
        None
    ));
    assert!(query_to_sql_equality::<Nullable<Citext>, Option<String>>(
        "NULL::citext",
        None
    ));
}

#[diesel_test_helper::test]
fn option_to_sql() {
    assert!(query_to_sql_equality::<Nullable<Integer>, Option<i32>>(
        "1",
        Some(1)
    ));
    assert!(query_to_sql_equality::<Nullable<Integer>, Option<i32>>(
        "NULL", None
    ));
    assert!(query_to_sql_equality::<Nullable<VarChar>, Option<String>>(
        "'Hello!'",
        Some("Hello!".to_string())
    ));
    assert!(query_to_sql_equality::<Nullable<VarChar>, Option<String>>(
        "''",
        Some("".to_string())
    ));
    assert!(query_to_sql_equality::<Nullable<VarChar>, Option<String>>(
        "NULL", None
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_array_from_sql() {
    assert_eq!(
        vec![true, false, true],
        query_single_value::<Array<Bool>, Vec<bool>>("ARRAY['t', 'f', 't']::bool[]")
    );
    assert_eq!(
        vec![1, 2, 3],
        query_single_value::<Array<Integer>, Vec<i32>>("ARRAY[1, 2, 3]")
    );
    assert_eq!(
        vec!["Hello".to_string(), "".to_string(), "world".to_string()],
        query_single_value::<Array<VarChar>, Vec<String>>("ARRAY['Hello', '', 'world']")
    );
}

#[cfg(feature = "postgres")]
#[diesel_test_helper::test]
fn pg_array_from_sql_non_one_lower_bound() {
    assert_eq!(
        vec![true, false, true],
        query_single_value::<Array<Bool>, Vec<bool>>("'[0:2]={t, f, t}'::bool[]")
    );
    assert_eq!(
        vec![true, false, true],
        query_single_value::<Array<Bool>, Vec<bool>>("'[1:3]={t, f, t}'::bool[]")
    );
    assert_eq!(
        vec![true, false, true],
        query_single_value::<Array<Bool>, Vec<bool>>("'[2:4]={t, f, t}'::bool[]")
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn to_sql_array() {
    assert!(query_to_sql_equality::<Array<Bool>, Vec<bool>>(
        "ARRAY['t', 'f', 't']::bool[]",
        vec![true, false, true]
    ));
    assert!(query_to_sql_equality::<Array<Bool>, &[bool]>(
        "ARRAY['t', 'f', 't']::bool[]",
        &[true, false, true]
    ));
    assert!(!query_to_sql_equality::<Array<Bool>, &[bool]>(
        "ARRAY['t', 'f', 't']::bool[]",
        &[false, false, true]
    ));
    assert!(query_to_sql_equality::<Array<Integer>, &[i32]>(
        "ARRAY[1, 2, 3]",
        &[1, 2, 3]
    ));
    assert!(query_to_sql_equality::<Array<VarChar>, &[&str]>(
        "ARRAY['Hello', '', 'world']::text[]",
        &["Hello", "", "world"]
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_array_containing_null() {
    let query = "ARRAY['Hello', '', NULL, 'world']";
    let data = query_single_value::<Array<Nullable<VarChar>>, Vec<Option<String>>>(query);
    let expected = vec![
        Some("Hello".to_string()),
        Some("".to_string()),
        None,
        Some("world".to_string()),
    ];
    assert_eq!(expected, data);
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn timestamp_from_sql() {
    use diesel::data_types::PgTimestamp;

    let query = "'2015-11-13 13:26:48.041057-07'::timestamp";
    let expected_value = PgTimestamp(500_736_408_041_057);
    assert_eq!(
        expected_value,
        query_single_value::<Timestamp, PgTimestamp>(query)
    );
    let query = "'2015-11-13 13:26:49.041057-07'::timestamp";
    let expected_value = PgTimestamp(500_736_409_041_057);
    assert_eq!(
        expected_value,
        query_single_value::<Timestamp, PgTimestamp>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_timestamp_to_sql_timestamp() {
    use diesel::data_types::PgTimestamp;

    let expected_value = "'2015-11-13 13:26:48.041057-07'::timestamp";
    let value = PgTimestamp(500_736_408_041_057);
    assert!(query_to_sql_equality::<Timestamp, PgTimestamp>(
        expected_value,
        value
    ));
    let expected_value = "'2015-11-13 13:26:49.041057-07'::timestamp";
    let value = PgTimestamp(500_736_409_041_057);
    assert!(query_to_sql_equality::<Timestamp, PgTimestamp>(
        expected_value,
        value
    ));
    let expected_non_equal_value = "'2015-11-13 13:26:48.041057-07'::timestamp";
    assert!(!query_to_sql_equality::<Timestamp, PgTimestamp>(
        expected_non_equal_value,
        value
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_numeric_from_sql() {
    use diesel::data_types::PgNumeric;

    let query = "1.0::numeric";
    let expected_value = PgNumeric::Positive {
        digits: vec![1],
        weight: 0,
        scale: 1,
    };
    assert_eq!(
        expected_value,
        query_single_value::<Numeric, PgNumeric>(query)
    );
    let query = "-31.0::numeric";
    let expected_value = PgNumeric::Negative {
        digits: vec![31],
        weight: 0,
        scale: 1,
    };
    assert_eq!(
        expected_value,
        query_single_value::<Numeric, PgNumeric>(query)
    );
    let query = "'NaN'::numeric";
    let expected_value = PgNumeric::NaN;
    assert_eq!(
        expected_value,
        query_single_value::<Numeric, PgNumeric>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_numeric_bigdecimal_to_sql() {
    use self::bigdecimal::BigDecimal;

    fn correct_rep(integer: u64, decimal: u64) -> bool {
        let expected = format!("{}.{}", integer, decimal);
        let value: BigDecimal = expected.parse().expect("Could not parse to a BigDecimal");
        query_to_sql_equality::<Numeric, BigDecimal>(&expected, value)
    }

    quickcheck(correct_rep as fn(u64, u64) -> bool);

    let test_values = vec![
        "0.1",
        "1.0",
        "141.0",
        "-1.0",
        // Larger than u64
        "18446744073709551616",
        // Powers of 10k (numeric is represented in base 10k)
        "10000",
        "100000000",
        "1.100001",
        "10000.100001",
        "0.00001234",
        "120000.00001234",
        "120001.00001234",
    ];

    for value in test_values {
        let expected = format!("'{}'::numeric", value);
        let value = value.parse::<BigDecimal>().unwrap();
        query_to_sql_equality::<Numeric, _>(&expected, value);
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn mysql_numeric_bigdecimal_to_sql() {
    use self::bigdecimal::BigDecimal;

    fn correct_rep(integer: u64, decimal: u64) -> bool {
        let expected = format!("{}.{}", integer, decimal);
        let value: BigDecimal = expected.parse().expect("Could not parse to a BigDecimal");
        query_to_sql_equality::<Numeric, BigDecimal>(&expected, value)
    }

    quickcheck(correct_rep as fn(u64, u64) -> bool);

    let test_values = vec![
        "1.0",
        "141.0",
        "-1.0",
        "10000",
        "100000000",
        "1.100001",
        "10000.100001",
        "0.00001234",
        "120000.00001234",
        "120001.00001234",
    ];

    for value in test_values {
        let expected = format!("cast('{}' as decimal(20, 10))", value);
        let value = value.parse::<BigDecimal>().unwrap();
        query_to_sql_equality::<Numeric, _>(&expected, value);
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_numeric_bigdecimal_from_sql() {
    use self::bigdecimal::BigDecimal;

    let values = vec![
        "0.1",
        "1.0",
        "141.0",
        "-1.0",
        // With some more precision
        "4.2000000",
        // Larger than u64
        "18446744073709551616",
        // Powers of 10k (numeric is represented in base 10k)
        "10000",
        "100000000",
        "1.100001",
        "10000.100001",
        "0.00001234",
        "120000.00001234",
        "120001.00001234",
    ];

    for value in values {
        let query = format!("'{}'::numeric", value);
        let expected = value.parse::<BigDecimal>().unwrap();
        assert_eq!(expected, query_single_value::<Numeric, BigDecimal>(&query));
        assert_eq!(
            format!("{}", expected),
            format!("{}", query_single_value::<Numeric, BigDecimal>(&query))
        );
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn mysql_numeric_bigdecimal_from_sql() {
    use self::bigdecimal::BigDecimal;

    let query = "cast(1.0 as decimal)";
    let expected_value: BigDecimal = "1.0".parse().expect("Could not parse to a BigDecimal");
    assert_eq!(
        expected_value,
        query_single_value::<Numeric, BigDecimal>(query)
    );

    let query = "cast(141.00 as decimal)";
    let expected_value: BigDecimal = "141.00".parse().expect("Could not parse to a BigDecimal");
    assert_eq!(
        expected_value,
        query_single_value::<Numeric, BigDecimal>(query)
    );

    // Some non standard values:
    let query = "cast(18446744073709551616 as decimal)"; // 2^64; doesn't fit in u64
                                                         // It is mysql, it will trim it even in strict mode
    let expected_value: BigDecimal = "9999999999.00"
        .parse()
        .expect("Could not parse to a BigDecimal");
    assert_eq!(
        expected_value,
        query_single_value::<Numeric, BigDecimal>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_uuid_from_sql() {
    extern crate uuid;

    let query = "'8a645207-42d6-4d17-82e7-f5e42ede0f67'::uuid";
    let expected_value = uuid::Uuid::parse_str("8a645207-42d6-4d17-82e7-f5e42ede0f67").unwrap();
    assert_eq!(
        expected_value,
        query_single_value::<Uuid, uuid::Uuid>(query)
    );
    let query = "'f94e0e4d-c7b0-405f-9c0e-57b97f4afb58'::uuid";
    let expected_value = uuid::Uuid::parse_str("f94e0e4d-c7b0-405f-9c0e-57b97f4afb58").unwrap();
    assert_eq!(
        expected_value,
        query_single_value::<Uuid, uuid::Uuid>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_uuid_to_sql_uuid() {
    extern crate uuid;

    let expected_value = "'8a645207-42d6-4d17-82e7-f5e42ede0f67'::uuid";
    let value = uuid::Uuid::parse_str("8a645207-42d6-4d17-82e7-f5e42ede0f67").unwrap();
    assert!(query_to_sql_equality::<Uuid, uuid::Uuid>(
        expected_value,
        value
    ));
    let expected_value = "'f94e0e4d-c7b0-405f-9c0e-57b97f4afb58'::uuid";
    let value = uuid::Uuid::parse_str("f94e0e4d-c7b0-405f-9c0e-57b97f4afb58").unwrap();
    assert!(query_to_sql_equality::<Uuid, uuid::Uuid>(
        expected_value,
        value
    ));
    let expected_non_equal_value = "'8e940686-97a5-4e8b-ac44-64cf3cceea9b'::uuid";
    assert!(!query_to_sql_equality::<Uuid, uuid::Uuid>(
        expected_non_equal_value,
        value
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_macaddress_from_sql() {
    let query = "'08:00:2b:01:02:03'::macaddr";
    let expected_value = [0x08, 0x00, 0x2b, 0x01, 0x02, 0x03];
    assert_eq!(
        expected_value,
        query_single_value::<MacAddr, [u8; 6]>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_macaddress_to_sql_macaddress() {
    let expected_value = "'08:00:2b:01:02:03'::macaddr";
    let value = [0x08, 0x00, 0x2b, 0x01, 0x02, 0x03];
    assert!(query_to_sql_equality::<MacAddr, [u8; 6]>(
        expected_value,
        value
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_macaddress8_from_sql() {
    let query = "'08:00:2b:01:02:03:04:05'::macaddr8";
    let expected_value = [0x08, 0x00, 0x2b, 0x01, 0x02, 0x03, 0x04, 0x05];
    assert_eq!(
        expected_value,
        query_single_value::<MacAddr8, [u8; 8]>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_macaddress8_to_sql_macaddress() {
    let expected_value = "'08:00:2b:01:02:03:04:05'::macaddr8";
    let value = [0x08, 0x00, 0x2b, 0x01, 0x02, 0x03, 0x04, 0x05];
    assert!(query_to_sql_equality::<MacAddr8, [u8; 8]>(
        expected_value,
        value
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_lsn_from_sql() {
    let query = "'08002b01/02030405'::pg_lsn";
    let expected_value = diesel::pg::data_types::PgLsn(0x08002b0102030405);
    assert_eq!(
        expected_value,
        query_single_value::<PgLsn, diesel::pg::data_types::PgLsn>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_lsn_to_sql_lsn() {
    let expected_value = "'08002b01/02030405'::pg_lsn";
    let value = diesel::pg::data_types::PgLsn(0x08002b0102030405);
    assert!(query_to_sql_equality::<PgLsn, diesel::pg::data_types::PgLsn>(expected_value, value));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_v4address_from_sql() {
    extern crate ipnetwork;
    use std::str::FromStr;

    let query = "'192.168.1.0/24'::cidr";
    let expected_value =
        ipnetwork::IpNetwork::V4(ipnetwork::Ipv4Network::from_str("192.168.1.0/24").unwrap());
    assert_eq!(
        expected_value,
        query_single_value::<Cidr, ipnetwork::IpNetwork>(query)
    );
    let query = "'192.168.1.0/24'::inet";
    let expected_value =
        ipnetwork::IpNetwork::V4(ipnetwork::Ipv4Network::from_str("192.168.1.0/24").unwrap());
    assert_eq!(
        expected_value,
        query_single_value::<Inet, ipnetwork::IpNetwork>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_v4address_from_sql_ipnet() {
    use std::str::FromStr;

    let query = "'192.168.1.0/24'::cidr";
    let expected_value = ipnet::IpNet::V4(ipnet::Ipv4Net::from_str("192.168.1.0/24").unwrap());
    assert_eq!(
        expected_value,
        query_single_value::<Cidr, ipnet::IpNet>(query)
    );
    let query = "'192.168.1.0/24'::inet";
    let expected_value = ipnet::IpNet::V4(ipnet::Ipv4Net::from_str("192.168.1.0/24").unwrap());
    assert_eq!(
        expected_value,
        query_single_value::<Inet, ipnet::IpNet>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_v6address_from_sql() {
    extern crate ipnetwork;
    use std::str::FromStr;

    let query = "'2001:4f8:3:ba::/64'::cidr";
    let expected_value =
        ipnetwork::IpNetwork::V6(ipnetwork::Ipv6Network::from_str("2001:4f8:3:ba::/64").unwrap());
    assert_eq!(
        expected_value,
        query_single_value::<Cidr, ipnetwork::IpNetwork>(query)
    );
    let query = "'2001:4f8:3:ba::/64'::inet";
    let expected_value =
        ipnetwork::IpNetwork::V6(ipnetwork::Ipv6Network::from_str("2001:4f8:3:ba::/64").unwrap());
    assert_eq!(
        expected_value,
        query_single_value::<Inet, ipnetwork::IpNetwork>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_v6address_from_sql_ipnet() {
    use std::str::FromStr;

    let query = "'2001:4f8:3:ba::/64'::cidr";
    let expected_value = ipnet::IpNet::V6(ipnet::Ipv6Net::from_str("2001:4f8:3:ba::/64").unwrap());
    assert_eq!(
        expected_value,
        query_single_value::<Cidr, ipnet::IpNet>(query)
    );
    let query = "'2001:4f8:3:ba::/64'::inet";
    let expected_value = ipnet::IpNet::V6(ipnet::Ipv6Net::from_str("2001:4f8:3:ba::/64").unwrap());
    assert_eq!(
        expected_value,
        query_single_value::<Inet, ipnet::IpNet>(query)
    );
}
#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_v4address_to_sql_v4address() {
    extern crate ipnetwork;
    use std::str::FromStr;

    let expected_value = "'192.168.1'::cidr";
    let value =
        ipnetwork::IpNetwork::V4(ipnetwork::Ipv4Network::from_str("192.168.1.0/24").unwrap());
    assert!(query_to_sql_equality::<Cidr, ipnetwork::IpNetwork>(
        expected_value,
        value
    ));
    let expected_value = "'192.168.1.0/24'::inet";
    let value =
        ipnetwork::IpNetwork::V4(ipnetwork::Ipv4Network::from_str("192.168.1.0/24").unwrap());
    assert!(query_to_sql_equality::<Inet, ipnetwork::IpNetwork>(
        expected_value,
        value
    ));
    let expected_non_equal_value = "'192.168.1.0/23'::inet";
    assert!(!query_to_sql_equality::<Inet, ipnetwork::IpNetwork>(
        expected_non_equal_value,
        value
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_v4address_to_sql_v4address_ipnet() {
    use std::str::FromStr;

    let expected_value = "'192.168.1'::cidr";
    let value = ipnet::IpNet::V4(ipnet::Ipv4Net::from_str("192.168.1.0/24").unwrap());
    assert!(query_to_sql_equality::<Cidr, ipnet::IpNet>(
        expected_value,
        value
    ));
    let expected_value = "'192.168.1.0/24'::inet";
    let value = ipnet::IpNet::V4(ipnet::Ipv4Net::from_str("192.168.1.0/24").unwrap());
    assert!(query_to_sql_equality::<Inet, ipnet::IpNet>(
        expected_value,
        value
    ));
    let expected_non_equal_value = "'192.168.1.0/23'::inet";
    assert!(!query_to_sql_equality::<Inet, ipnet::IpNet>(
        expected_non_equal_value,
        value
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_v6address_to_sql_v6address() {
    extern crate ipnetwork;
    use std::str::FromStr;

    let expected_value = "'2001:4f8:3:ba::/64'::cidr";
    let value =
        ipnetwork::IpNetwork::V6(ipnetwork::Ipv6Network::from_str("2001:4f8:3:ba::/64").unwrap());
    assert!(query_to_sql_equality::<Cidr, ipnetwork::IpNetwork>(
        expected_value,
        value
    ));
    let expected_value = "'2001:4f8:3:ba::/64'::cidr";
    let value =
        ipnetwork::IpNetwork::V6(ipnetwork::Ipv6Network::from_str("2001:4f8:3:ba::/64").unwrap());
    assert!(query_to_sql_equality::<Inet, ipnetwork::IpNetwork>(
        expected_value,
        value
    ));
    let expected_non_equal_value = "'2001:4f8:3:ba::/63'::cidr";
    assert!(!query_to_sql_equality::<Inet, ipnetwork::IpNetwork>(
        expected_non_equal_value,
        value
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_v6address_to_sql_v6address_ipnet() {
    use std::str::FromStr;

    let expected_value = "'2001:4f8:3:ba::/64'::cidr";
    let value = ipnet::IpNet::V6(ipnet::Ipv6Net::from_str("2001:4f8:3:ba::/64").unwrap());
    assert!(query_to_sql_equality::<Cidr, ipnet::IpNet>(
        expected_value,
        value
    ));
    let expected_value = "'2001:4f8:3:ba::/64'::cidr";
    let value = ipnet::IpNet::V6(ipnet::Ipv6Net::from_str("2001:4f8:3:ba::/64").unwrap());
    assert!(query_to_sql_equality::<Inet, ipnet::IpNet>(
        expected_value,
        value
    ));
    let expected_non_equal_value = "'2001:4f8:3:ba::/63'::cidr";
    assert!(!query_to_sql_equality::<Inet, ipnet::IpNet>(
        expected_non_equal_value,
        value
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_json_from_sql() {
    extern crate serde_json;

    let query = "'true'::json";
    let expected_value = serde_json::Value::Bool(true);
    assert_eq!(
        expected_value,
        query_single_value::<Json, serde_json::Value>(query)
    );
}

// See https://stackoverflow.com/q/32843213/12089 for why we don't have a
// pg_json_to_sql_json test.  There's no `'true':json = 'true':json`
// because JSON string representations are ambiguous.  We _do_ have this
// test for `jsonb` values.

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_jsonb_from_sql() {
    extern crate serde_json;

    let query = "'true'::jsonb";
    let expected_value = serde_json::Value::Bool(true);
    assert_eq!(
        expected_value,
        query_single_value::<Jsonb, serde_json::Value>(query)
    );
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn pg_jsonb_to_sql_jsonb() {
    extern crate serde_json;

    let expected_value = "'false'::jsonb";
    let value = serde_json::Value::Bool(false);
    assert!(query_to_sql_equality::<Jsonb, serde_json::Value>(
        expected_value,
        value
    ));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn text_array_can_be_assigned_to_varchar_array_column() {
    let conn = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", conn);
    let post = insert_into(posts::table)
        .values(&sean.new_post("Hello", None))
        .get_result::<Post>(conn)
        .unwrap();

    update(posts::table.find(post.id))
        .set(posts::tags.eq(vec!["tag1", "tag2"]))
        .execute(conn)
        .unwrap();
    let tags_in_db = posts::table.find(post.id).select(posts::tags).first(conn);

    assert_eq!(Ok(vec!["tag1".to_string(), "tag2".to_string()]), tags_in_db);
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn third_party_crates_can_add_new_types() {
    use diesel::deserialize::FromSql;
    use diesel::pg::PgValue;

    #[derive(Debug, Clone, Copy, QueryId, SqlType)]
    struct MyInt;

    impl HasSqlType<MyInt> for Pg {
        fn metadata(lookup: &mut Self::MetadataLookup) -> Self::TypeMetadata {
            <Pg as HasSqlType<Integer>>::metadata(lookup)
        }
    }

    impl FromSql<MyInt, Pg> for i32 {
        fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
            FromSql::<Integer, Pg>::from_sql(bytes)
        }
    }

    assert_eq!(0, query_single_value::<MyInt, i32>("0"));
    assert_eq!(-1, query_single_value::<MyInt, i32>("-1"));
    assert_eq!(70_000, query_single_value::<MyInt, i32>("70000"));
}

fn query_single_value<T, U>(sql_str: &str) -> U
where
    U: FromSqlRow<T, TestBackend> + 'static,
    TestBackend: HasSqlType<T>,
    T: QueryId + SingleValue + SqlType,
{
    use diesel::dsl::sql;
    let connection = &mut connection();
    select(sql::<T>(sql_str)).first(connection).unwrap()
}

use diesel::expression::{is_aggregate, AsExpression, SqlLiteral, ValidGrouping};
use diesel::query_builder::{QueryFragment, QueryId};
use std::fmt::Debug;

fn query_to_sql_equality<T, U>(sql_str: &str, value: U) -> bool
where
    U: AsExpression<T> + Debug + Clone,
    U::Expression: SelectableExpression<diesel::internal::table_macro::NoFromClause, SqlType = T>
        + ValidGrouping<(), IsAggregate = is_aggregate::Never>,
    U::Expression: QueryFragment<TestBackend> + QueryId,
    T: QueryId + SingleValue + SqlType,
    <T as diesel::sql_types::SqlType>::IsNull:
        diesel::sql_types::OneIsNullable<<T as diesel::sql_types::SqlType>::IsNull>,
    <<T as diesel::sql_types::SqlType>::IsNull as diesel::sql_types::OneIsNullable<
        <T as diesel::sql_types::SqlType>::IsNull,
    >>::Out: diesel::sql_types::MaybeNullableType<diesel::sql_types::Bool>,
    diesel::dsl::Nullable<diesel::dsl::Eq<SqlLiteral<T>, U>>:
        Expression<SqlType = diesel::sql_types::Nullable<diesel::sql_types::Bool>>,
{
    use diesel::dsl::sql;
    let connection = &mut connection();
    let query = select(
        sql::<T>(sql_str)
            .is_null()
            .and(value.clone().as_expression().is_null())
            .or(sql::<T>(sql_str).eq(value.clone()).nullable()),
    );
    query
        .get_result::<Option<bool>>(connection)
        .expect(&format!("Error comparing {}, {:?}", sql_str, value))
        .unwrap_or(false)
}

#[cfg(feature = "postgres")]
#[diesel_test_helper::test]
#[should_panic(expected = "Received more than 4 bytes while decoding an i32")]
fn debug_check_catches_reading_bigint_as_i32_when_using_raw_sql() {
    use diesel::dsl::sql;
    use diesel::sql_types::Integer;

    let connection = &mut connection();
    users::table
        .select(sql::<Integer>("COUNT(*)"))
        .get_result::<i32>(connection)
        .unwrap();
}

#[cfg(feature = "postgres")]
#[diesel_test_helper::test]
fn test_range_from_sql() {
    use diesel::dsl::sql;
    use std::collections::Bound;

    let connection = &mut connection();

    let query = "'[1,)'::int4range";
    let expected_value = (Bound::Included(1), Bound::Unbounded);
    assert_eq!(
        expected_value,
        query_single_value::<Range<Int4>, (Bound<i32>, Bound<i32>)>(query)
    );

    let query = "'(1,2]'::int4range";
    let expected_value = (Bound::Included(2), Bound::Excluded(3));
    assert_eq!(
        expected_value,
        query_single_value::<Range<Int4>, (Bound<i32>, Bound<i32>)>(query)
    );

    let query = "SELECT '[2,1)'::int4range";
    assert!(sql::<Range<Int4>>(query)
        .load::<(Bound<i32>, Bound<i32>)>(connection)
        .is_err());

    let query = "'empty'::int4range";
    let expected_value = (Bound::Excluded(0), Bound::Excluded(0));
    assert_eq!(
        expected_value,
        query_single_value::<Range<Int4>, (Bound<i32>, Bound<i32>)>(query)
    );

    let query = "'(1,1)'::int4range";
    let expected_value = (Bound::Excluded(0), Bound::Excluded(0));
    assert_eq!(
        expected_value,
        query_single_value::<Range<Int4>, (Bound<i32>, Bound<i32>)>(query)
    );

    let query = "'(1,1]'::int4range";
    let expected_value = (Bound::Excluded(0), Bound::Excluded(0));
    assert_eq!(
        expected_value,
        query_single_value::<Range<Int4>, (Bound<i32>, Bound<i32>)>(query)
    );

    let query = "'[1,1)'::int4range";
    let expected_value = (Bound::Excluded(0), Bound::Excluded(0));
    assert_eq!(
        expected_value,
        query_single_value::<Range<Int4>, (Bound<i32>, Bound<i32>)>(query)
    );

    let query = "'[1,1]'::int4range";
    let expected_value = (Bound::Included(1), Bound::Excluded(2));
    assert_eq!(
        expected_value,
        query_single_value::<Range<Int4>, (Bound<i32>, Bound<i32>)>(query)
    );
}

#[cfg(feature = "postgres")]
#[diesel_test_helper::test]
fn test_range_to_sql() {
    use std::collections::Bound;

    let expected_value = "'[1,2]'::int4range";
    let value = (Bound::Included(1), Bound::Included(2));
    assert!(query_to_sql_equality::<Range<Int4>, (Bound<i32>, Bound<i32>)>(expected_value, value));
    let value = 1..=2;
    assert!(query_to_sql_equality::<
        Range<Int4>,
        std::ops::RangeInclusive<i32>,
    >(expected_value, value));

    let expected_value = "'(1,2]'::int4range";
    let value = (Bound::Excluded(1), Bound::Included(2));
    assert!(query_to_sql_equality::<Range<Int4>, (Bound<i32>, Bound<i32>)>(expected_value, value));

    let expected_value = "'[1,2)'::int4range";
    let value = (Bound::Included(1), Bound::Excluded(2));
    assert!(query_to_sql_equality::<Range<Int4>, (Bound<i32>, Bound<i32>)>(expected_value, value));
    let value = 1..2;
    assert!(query_to_sql_equality::<Range<Int4>, std::ops::Range<i32>>(
        expected_value,
        value
    ));

    let expected_value = "'(,2)'::int4range";
    let value = (Bound::Unbounded, Bound::Excluded(2));
    assert!(query_to_sql_equality::<Range<Int4>, (Bound<i32>, Bound<i32>)>(expected_value, value));
    let value = ..2;
    assert!(query_to_sql_equality::<Range<Int4>, std::ops::RangeTo<i32>>(expected_value, value));

    let expected_value = "'(,2]'::int4range";
    let value = (Bound::Unbounded, Bound::Included(2));
    assert!(query_to_sql_equality::<Range<Int4>, (Bound<i32>, Bound<i32>)>(expected_value, value));
    let value = ..=2;
    assert!(query_to_sql_equality::<
        Range<Int4>,
        std::ops::RangeToInclusive<i32>,
    >(expected_value, value));

    let expected_value = "'[1,)'::int4range";
    let value = (Bound::Included(1), Bound::Unbounded);
    assert!(query_to_sql_equality::<Range<Int4>, (Bound<i32>, Bound<i32>)>(expected_value, value));
    let value = 1..;
    assert!(query_to_sql_equality::<Range<Int4>, std::ops::RangeFrom<i32>>(expected_value, value));
}

#[cfg(feature = "postgres")]
#[diesel_test_helper::test]
fn test_range_bound_enum_to_sql() {
    assert!(query_to_sql_equality::<RangeBoundEnum, RangeBound>(
        "'[]'",
        RangeBound::LowerBoundInclusiveUpperBoundInclusive
    ));
    assert!(query_to_sql_equality::<RangeBoundEnum, RangeBound>(
        "'[)'",
        RangeBound::LowerBoundInclusiveUpperBoundExclusive
    ));
    assert!(query_to_sql_equality::<RangeBoundEnum, RangeBound>(
        "'(]'",
        RangeBound::LowerBoundExclusiveUpperBoundInclusive
    ));
    assert!(query_to_sql_equality::<RangeBoundEnum, RangeBound>(
        "'()'",
        RangeBound::LowerBoundExclusiveUpperBoundExclusive
    ));
}

#[cfg(feature = "postgres")]
#[diesel_test_helper::test]
fn test_multirange_from_sql() {
    use diesel::dsl::sql;
    use std::collections::Bound;

    let connection = &mut connection();

    let query = "'{(,1), [5,8), [10,)}'::int4multirange";
    let expected_value = vec![
        (Bound::Unbounded, Bound::Excluded(1)),
        (Bound::Included(5), Bound::Excluded(8)),
        (Bound::Included(10), Bound::Unbounded),
    ];
    assert_eq!(
        expected_value,
        query_single_value::<Multirange<Int4>, Vec<(Bound<i32>, Bound<i32>)>>(query)
    );
}

#[cfg(feature = "postgres")]
#[diesel_test_helper::test]
fn test_multirange_to_sql() {
    use diesel::dsl::sql;
    use std::collections::Bound;

    let expected_value = "'{(,1), [5,8), [10,)}'::int4multirange";
    let value = vec![
        (Bound::Unbounded, Bound::Excluded(1)),
        (Bound::Included(5), Bound::Excluded(8)),
        (Bound::Included(10), Bound::Unbounded),
    ];
    assert!(query_to_sql_equality::<
        Multirange<Int4>,
        Vec<(Bound<i32>, Bound<i32>)>,
    >(expected_value, value));

    let expected_value = "'{[5,8)}'::int4multirange";
    let value = vec![5..8];
    assert!(query_to_sql_equality::<
        Multirange<Int4>,
        Vec<(std::ops::Range<i32>)>,
    >(expected_value, value));
}

#[cfg(feature = "postgres")]
#[diesel_test_helper::test]
fn test_inserting_ranges() {
    use std::collections::Bound;

    let connection = &mut connection();
    diesel::sql_query(
        "CREATE TABLE has_ranges (
                        id SERIAL PRIMARY KEY,
                        nul_range INT4RANGE,
                        range INT4RANGE NOT NULL)",
    )
    .execute(connection)
    .unwrap();
    table!(
        has_ranges(id) {
            id -> Int4,
            nul_range -> Nullable<Range<Int4>>,
            range -> Range<Int4>,
        }
    );

    let value = (Bound::Included(1), Bound::Excluded(3));

    let (_, v1, v2): (i32, Option<(_, _)>, (_, _)) = insert_into(has_ranges::table)
        .values((has_ranges::nul_range.eq(value), has_ranges::range.eq(value)))
        .get_result(connection)
        .unwrap();
    assert_eq!(v1, Some(value));
    assert_eq!(v2, value);
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn cchar_from_sql() {
    assert_eq!(b'\xc6', query_single_value::<CChar, u8>(r#"'Ǝ'::"char""#)); // postgres always uses the utf8 lower byte, Ǝ utf8 is 0xC6 0x8E
    assert_eq!(b'a', query_single_value::<CChar, u8>(r#"'a'::"char""#));
    assert_eq!(b' ', query_single_value::<CChar, u8>(r#"' '::"char""#));
    assert_eq!(b'~', query_single_value::<CChar, u8>(r#"'~'::"char""#));
    assert_eq!(b'5', query_single_value::<CChar, u8>(r#"'5'::"char""#));
    assert_eq!(b'\\', query_single_value::<CChar, u8>(r#"'\'::"char""#));
    assert_eq!(b'\"', query_single_value::<CChar, u8>(r#"'"'::"char""#));
    assert_eq!(b'\'', query_single_value::<CChar, u8>(r#"''''::"char""#));
    assert_eq!(b'`', query_single_value::<CChar, u8>(r#"'`'::"char""#));
    assert_eq!(195u8, query_single_value::<CChar, u8>(r#"'ö'::"char""#));
    assert_eq!(195u8, query_single_value::<CChar, u8>(r#"'ä'::"char""#));
    assert_eq!(b'\0', query_single_value::<CChar, u8>(r#"0::"char""#));
    assert_eq!(b'0', query_single_value::<CChar, u8>(r#"'0'::"char""#));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn cchar_to_sql() {
    assert!(query_to_sql_equality::<CChar, u8>(
        r#"'Ǝ'::"char""#,
        b'\xc6'
    )); // postgres always uses the utf8 lower byte, Ǝ utf8 is 0xC6 0x8E
    assert!(query_to_sql_equality::<CChar, u8>(r#"'a'::"char""#, b'a'));
    assert!(query_to_sql_equality::<CChar, u8>(r#"' '::"char""#, b' '));
    assert!(query_to_sql_equality::<CChar, u8>(r#"'~'::"char""#, b'~'));
    assert!(query_to_sql_equality::<CChar, u8>(r#"'5'::"char""#, b'5'));
    assert!(query_to_sql_equality::<CChar, u8>(r#"'\'::"char""#, b'\\'));
    assert!(query_to_sql_equality::<CChar, u8>(r#"'"'::"char""#, b'\"'));
    assert!(query_to_sql_equality::<CChar, u8>(r#"''''::"char""#, b'\''));
    assert!(query_to_sql_equality::<CChar, u8>(r#"'`'::"char""#, b'`'));
    assert!(query_to_sql_equality::<CChar, u8>(r#"'ö'::"char""#, 195u8));
    assert!(query_to_sql_equality::<CChar, u8>(r#"'ä'::"char""#, 195u8));
    assert!(query_to_sql_equality::<CChar, u8>(r#"0::"char""#, b'\0'));
    assert!(query_to_sql_equality::<CChar, u8>(r#"'0'::"char""#, b'0'));
}

#[cfg(feature = "postgres")]
#[diesel_test_helper::test]
fn citext_fields() {
    let connection = &mut connection();

    // Enable the CIText extension
    diesel::sql_query("CREATE EXTENSION IF NOT EXISTS citext")
        .execute(connection)
        .unwrap();

    diesel::sql_query(
        "CREATE TABLE case_insensitive (
            id SERIAL PRIMARY KEY,
            non_null_ci citext NOT NULL,
            nullable_ci citext NULL,
            null_value citext NULL)",
    )
    .execute(connection)
    .unwrap();

    table! {
        case_insensitive (id) {
            id -> Int4,
            non_null_ci -> Citext,
            nullable_ci -> Nullable<Citext>,
            null_value -> Nullable<Citext>,
        }
    }

    let rows_inserted = insert_into(case_insensitive::table)
        .values((
            case_insensitive::non_null_ci.eq("UPPERCASE_VALUE".to_string()),
            case_insensitive::nullable_ci.eq("lowercase_value"),
            // Explicitly insert NULL
            case_insensitive::null_value.eq(None::<String>.into_sql()),
        ))
        .execute(connection)
        .unwrap();

    // Demonstrates that a query for the uppercase value in the database will succeed when
    // the search value is the lower cased version of that value
    // Also verifies that a null value can be deserialised
    let (uppercase_in_db, null_value): (String, Option<String>) = case_insensitive::table
        .filter(case_insensitive::non_null_ci.eq("UPPERCASE_VALUE".to_lowercase()))
        .select((case_insensitive::non_null_ci, case_insensitive::null_value))
        .first(connection)
        .unwrap();

    assert_eq!(uppercase_in_db, "UPPERCASE_VALUE");
    assert_eq!(null_value, Option::None);

    // Demonstrates that a query for the lowercase value in the database will succeed when
    // the search value is the upper cased version of that value
    let (lowercase_in_db): (Option<String>) = case_insensitive::table
        .filter(case_insensitive::nullable_ci.eq("lowercase_value".to_uppercase()))
        .select((case_insensitive::nullable_ci))
        .first(connection)
        .unwrap();

    assert_eq!(lowercase_in_db, Some("lowercase_value".to_string()));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn deserialize_wrong_primitive_gives_good_error() {
    let conn = &mut connection();

    diesel::sql_query(
        "CREATE TABLE test_table(\
                       bool BOOLEAN,
                       small SMALLINT, \
                       int INTEGER, \
                       big BIGINT, \
                       float FLOAT4, \
                       double FLOAT8,
                       text TEXT)",
    )
    .execute(conn)
    .unwrap();
    diesel::sql_query("INSERT INTO test_table VALUES('t', 1, 1, 1, 1, 1, 'long text long text')")
        .execute(conn)
        .unwrap();

    let res = diesel::dsl::sql::<SmallInt>("SELECT bool FROM test_table").get_result::<i16>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'bool': \
         Received less than 2 bytes while decoding an i16. \
         Was an expression of a different type accidentally marked as SmallInt?"
    );

    let res = diesel::dsl::sql::<SmallInt>("SELECT int FROM test_table").get_result::<i16>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'int': \
         Received more than 2 bytes while decoding an i16. \
         Was an Integer expression accidentally marked as SmallInt?"
    );

    let res = diesel::dsl::sql::<Integer>("SELECT small FROM test_table").get_result::<i32>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'small': \
         Received less than 4 bytes while decoding an i32. \
         Was an SmallInt expression accidentally marked as Integer?"
    );

    let res = diesel::dsl::sql::<Integer>("SELECT big FROM test_table").get_result::<i32>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'big': \
         Received more than 4 bytes while decoding an i32. \
         Was an BigInt expression accidentally marked as Integer?"
    );

    let res = diesel::dsl::sql::<BigInt>("SELECT int FROM test_table").get_result::<i64>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'int': \
         Received less than 8 bytes while decoding an i64. \
         Was an Integer expression accidentally marked as BigInt?"
    );

    let res = diesel::dsl::sql::<BigInt>("SELECT text FROM test_table").get_result::<i64>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'text': \
         Received more than 8 bytes while decoding an i64. \
         Was an expression of a different type expression accidentally marked as BigInt?"
    );

    let res = diesel::dsl::sql::<Float>("SELECT small FROM test_table").get_result::<f32>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'small': \
         Received less than 4 bytes while decoding an f32. \
         Was a numeric accidentally marked as float?"
    );

    let res = diesel::dsl::sql::<Float>("SELECT double FROM test_table").get_result::<f32>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'double': \
         Received more than 4 bytes while decoding an f32. \
         Was a double accidentally marked as float?"
    );

    let res = diesel::dsl::sql::<Double>("SELECT float FROM test_table").get_result::<f64>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'float': \
         Received less than 8 bytes while decoding an f64. \
         Was a float accidentally marked as double?"
    );

    let res = diesel::dsl::sql::<Double>("SELECT text FROM test_table").get_result::<f64>(conn);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Error deserializing field 'text': \
         Received more than 8 bytes while decoding an f64. \
         Was a numeric accidentally marked as double?"
    );
}
