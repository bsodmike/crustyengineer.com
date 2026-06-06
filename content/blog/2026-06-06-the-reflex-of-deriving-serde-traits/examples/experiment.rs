use crate::core::{CalculationsDO, InterestTotal};

use rust_decimal::Decimal;

mod core {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_with::{DeserializeAs, SerializeAs};

    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq)]
    pub struct InterestTotal(pub Decimal);

    #[derive(Clone)]
    pub struct CalculationsDO {
        pub interest: InterestTotal,
    }

    pub struct InterestTotalAsDecimal;

    impl SerializeAs<InterestTotal> for InterestTotalAsDecimal {
        fn serialize_as<S: Serializer>(
            source: &InterestTotal,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            source.0.to_string().serialize(serializer)
        }
    }

    impl<'de> DeserializeAs<'de, InterestTotal> for InterestTotalAsDecimal {
        fn deserialize_as<D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<InterestTotal, D::Error> {
            let s = String::deserialize(deserializer)?;

            Ok(InterestTotal(
                Decimal::from_str(&s).map_err(serde::de::Error::custom)?,
            ))
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
        #[serde_as(as = "core::InterestTotalAsDecimal")]
        interest: core::InterestTotal,
    }

    impl From<core::CalculationsDO> for DomainObjectDb {
        fn from(value: core::CalculationsDO) -> Self {
            Self {
                interest: value.interest,
            }
        }
    }

    impl From<DomainObjectDb> for core::CalculationsDO {
        fn from(value: DomainObjectDb) -> Self {
            Self {
                interest: value.interest,
            }
        }
    }
}

fn main() {
    let core_obj = core::CalculationsDO {
        interest: core::InterestTotal(Decimal::new(100, 0)),
    };

    // serialize
    let db_obj: db::DomainObjectDb = core_obj.clone().into();
    assert_eq!(
        serde_json::to_string_pretty(&db_obj).unwrap(),
        r#"{
  "interest": "100"
}"#
    );

    // deserialize
    let json = r#"
    {
        "interest": "42"
    }
    "#;
    let obj: db::DomainObjectDb = serde_json::from_str(json).unwrap();
    let converted: CalculationsDO = obj.into();
    assert_eq!(converted.interest, InterestTotal(Decimal::new(42, 0)));
}
