mod core {
    use chrono::DateTime;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_with::{DeserializeAs, SerializeAs};

    #[derive(Clone)]
    pub struct UnixTimestamp(pub u64);

    #[derive(Clone)]
    pub struct DomainObject {
        pub name: String,
        pub unix_ts: UnixTimestamp,
    }

    pub struct UnixTsAsRfc3339;

    impl SerializeAs<UnixTimestamp> for UnixTsAsRfc3339 {
        fn serialize_as<S: Serializer>(
            source: &UnixTimestamp,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let dt = DateTime::from_timestamp_nanos(source.0 as i64);
            dt.to_rfc3339().serialize(serializer)
        }
    }

    impl<'de> DeserializeAs<'de, UnixTimestamp> for UnixTsAsRfc3339 {
        fn deserialize_as<D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<UnixTimestamp, D::Error> {
            let s = String::deserialize(deserializer)?;
            let dt = DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
            let ns = dt
                .timestamp_nanos_opt()
                .filter(|&n| n >= 0)
                .ok_or_else(|| serde::de::Error::custom("timestamps out of range"))?;
            Ok(UnixTimestamp(ns as u64))
        }
    }

    pub struct UnixTsAsMillis;

    impl SerializeAs<UnixTimestamp> for UnixTsAsMillis {
        fn serialize_as<S: Serializer>(
            source: &UnixTimestamp,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            (source.0 / 1_000_000).serialize(serializer)
        }
    }

    impl<'de> DeserializeAs<'de, UnixTimestamp> for UnixTsAsMillis {
        fn deserialize_as<D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<UnixTimestamp, D::Error> {
            let ms = u64::deserialize(deserializer)?;
            Ok(UnixTimestamp(ms * 1_000_000))
        }
    }
}

mod fe {
    use super::core;

    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    pub struct DomainObjectFe {
        name: String,
        #[serde_as(as = "Option<core::UnixTsAsRfc3339>")]
        unix_ts: Option<core::UnixTimestamp>,
    }

    impl From<core::DomainObject> for DomainObjectFe {
        fn from(value: core::DomainObject) -> Self {
            Self {
                name: value.name,
                unix_ts: Some(value.unix_ts),
            }
        }
    }
}

mod db {
    use super::core;

    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    pub struct DomainObjectDb {
        name: String,
        #[serde_as(as = "core::UnixTsAsMillis")]
        unix_ts: core::UnixTimestamp,
    }

    impl From<core::DomainObject> for DomainObjectDb {
        fn from(value: core::DomainObject) -> Self {
            Self {
                name: value.name,
                unix_ts: value.unix_ts,
            }
        }
    }
}

fn main() {
    let core_obj = core::DomainObject {
        name: "hello".to_string(),
        unix_ts: core::UnixTimestamp(1_700_000_000_000_000_000),
    };

    let fe_obj: fe::DomainObjectFe = core_obj.clone().into();
    let db_obj: db::DomainObjectDb = core_obj.into();

    assert_eq!(
        serde_json::to_string_pretty(&fe_obj).unwrap(),
        r#"{
  "name": "hello",
  "unix_ts": "2023-11-14T22:13:20+00:00"
}"#
    );

    assert_eq!(
        serde_json::to_string_pretty(&db_obj).unwrap(),
        r#"{
  "name": "hello",
  "unix_ts": 1700000000000
}"#
    );
}
