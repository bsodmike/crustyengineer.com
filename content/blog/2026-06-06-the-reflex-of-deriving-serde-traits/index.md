+++
title = "The Reflex of Deriving `serde` Traits"

[taxonomies]
tags = ["rust"] 
+++

This popped up in "This Week in Rust" #653 by [Evgeniy Terekhin](https://verrchu.github.io/blog/3-the-reflex-of-deriving-serde-traits/) ([verrchu](https://github.com/verrchu)).

The
`DomainObject` type is mentioned in the article however one must refer to the full source to see its
`From` conversion, as I have included below

```rust
impl From<core::DomainObject> for DomainObjectFe {
    fn from(value: core::DomainObject) -> Self {
        Self {
            name: value.name,
            unix_ts: Some(value.unix_ts),
        }
    }
}
```

This lets you call
`.into()` to perform the value conversion. Since it is an owned value, it is also cloned as it is used twice in this particular example.

```rust
fn example() {
    let core_obj = core::DomainObject {
        name: "hello".to_string(),
        unix_ts: core::UnixTimestamp(1_700_000_000_000_000_000),
    };

    let fe_obj: fe::DomainObjectFe = core_obj.clone().into();
    let db_obj: db::DomainObjectDb = core_obj.into();
}
```

# Demonstrating deserialization (`deser`)

In my ledger system, I am using `rust_decimal`'s `Decimal` type and value conversions are either manual or implementing
`From`. Full accompanying source code can be found [here](https://github.com/bsodmike/crustyengineer.com/tree/master/content/blog/2026-06-06-the-reflex-of-deriving-serde-traits/examples/experiment.rs) to demonstrate deserialization.

Note how we are deserializing from a `String` value of `"42"` to `Decimal`.

```rust
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
```