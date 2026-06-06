+++
title = "The reflex of deriving `serde` traits"

[taxonomies]
tags = ["rust"] 
+++

This popped up in "This Week in Rust" #653 by [Evgeniy Terekhin](https://verrchu.github.io/blog/3-the-reflex-of-deriving-serde-traits/) ([verrchu](https://github.com/verrchu)).

This is a tiny addendum to those new to Rust, and may be confused by the
`DomainObject` type; it's mentioned in the article but not referenced by any code.

You will however find its use in the actual code linked where it's used to implement
`From` for type conversions. This is made clear in the example source itself

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

This lets you call `.into()`. The only reason this is cloned below is simply due to being used twice and the
`From` conversion is only implemented on an owned value.

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
