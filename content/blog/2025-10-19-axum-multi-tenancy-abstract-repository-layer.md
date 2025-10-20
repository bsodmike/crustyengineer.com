+++
title = "Axum: Multi-tenancy (with Hexarch) and Abstracting the Repository Layer"
# description = ""

[taxonomies]
tags = [ "rust", "hexarch", "axum"] 
+++

Implementing a repository layer generally looks like

```rs
impl UserRepository for PostgresDBOutbound {
    async fn create(&self, creation: UserCreation) -> anyhow::Result<UserInfo> {
        let user = sqlx::query_as!(
            UserInfo,
            r#"
            INSERT INTO "users" (
                "email"
            ) VALUES ($1)
            RETURNING
                "id",
                "email",
                "firstname",
                "lastname",
                "created_at",
                "updated_at"
            "#,
            creation.email
        )
        .fetch_one(&self.pool)
        .await
        .context("could not create User")?;

        Ok(user)
    }

    //...
}
```

## `sqlx` has better multi-tenancy support (as of 5-days ago)

I noticed that `sqlx` had released [0.9.0-alpha.1](https://github.com/launchbadge/sqlx/blob/main/CHANGELOG.md#090-alpha1---2025-10-14) and mentions PR[#3383](https://github.com/launchbadge/sqlx/pull/3383) with a full example for setting up [multi-tenancy](https://github.com/launchbadge/sqlx/tree/main/examples/postgres/multi-tenant) with Postgres.

The above example's file-structure has some interesting aspects to note:

- the main binary has `src/migrations`. Each sub-crate has them above `src/lib.rs`.
- `sqlx.toml` appears in the main binary and each sub-crate.
- each sub-crate maps to a Postgres schema (namespace) -- `accounts.account` is the account table inside the `accounts` schema; it also has it's own `_sqlx_migrations` table.
- the postgres `public` schema has it's own `_sqlx_migrations` table as well.

Since accounts are a primary aspect, in the sense that not only are they are the "tenant" but also foreign-key constraints will tie back to it (in most cases), if you refer to the [example's README](https://github.com/launchbadge/sqlx/blob/main/examples/postgres/multi-tenant/README.md) -- migrations need to be run in a specific order:

1. run migrations in `accounts`
2. all other sub-crates (any order)
3. in the main binary

```
.
├── Cargo.toml
├── README.md
├── accounts
│   ├── Cargo.toml
│   ├── migrations
│   │   ├── 01_setup.sql
│   │   ├── 02_account.sql
│   │   └── 03_session.sql
│   ├── sqlx.toml
│   └── src
│       └── lib.rs
├── payments
│   ├── Cargo.toml
│   ├── migrations
│   │   ├── 01_setup.sql
│   │   └── 02_payment.sql
│   ├── sqlx.toml
│   └── src
│       └── lib.rs
├── sqlx.toml
└── src
    ├── main.rs
    └── migrations
        ├── 01_setup.sql
        └── 02_purchase.sql
```

## Another "abstraction" to the repository layer

[daymare](https://daymare.net/blogs/everbody-so-creative/) was just commenting on the need for constant abstractions, but this offers two benefits:

- the sub-crates only care about primitive types, as these map (more or less) to the primitive types supported by `sqlx` and their Postgres equivalents.
- as such sub-crate public APIs may not need to change types more often
- types used by the repository interface can change (as much as they need to), without impacting the sub-crate API.

Please note that I am using the term _abstraction_ here rather loosely. In a strict sense, this would refer to the use of generics and traits (which we do so later on, with feature gates). The repository layer is still the last abstraction layer of significance but we are now placing a crate boundary between this layer and our sqlx sub-crates -- which is also an abstraction in the sense of

- strict public API
- encapsulation
- organisation
- visibility (as needed!)

Now, I can share the stack of sub-crates with another engineer and they can have a full postgres step, which is decoupled from the host Axum application. We can design sub-systems with the knowledge that each tenant has an `Account` and even enforce foreign-key references (as needed).

Use of a transaction below is a placeholder, as we may create associated records in join-tables related to the account upon its creation.

```rs
impl UserRepository for PostgresDBOutbound {
    async fn create(&self, creation: UserCreation) -> anyhow::Result<UserInfo> {
        let accounts = AccountsManager::setup(&self.pool)
            .await
            .map_err(|err| anyhow!(err).context("error initializing AccountsManager"))?;

        // Use externally managed transaction, as an example only
        let mut txn = self.pool.begin().await?;

        let account_id = accounts.create(&mut txn, &creation.email).await?;
        // note: call to other sub-crates to create any records inside other postgres schema (as needed).

        txn.commit().await?;

        // skipped: convert to UserInfo, returned as `r` below

        Ok(r)
    }

    //...
}
```

This is the layout I have adopted, with a sub-crate called `postgres_interfaces` -- more on this below:

```
persistence
└── postgres
    ├── accounts
    │   ├── Cargo.toml
    │   ├── migrations
    │   │   ├── 01_setup.sql
    │   │   ├── 02_account.sql
    │   │   └── 03_session.sql
    │   ├── sqlx.toml
    │   └── src
    │       ├── lib.rs
    │       ├── models
    │       └── models.rs
    ├── interfaces
    │   ├── Cargo.toml
    │   └── src
    │       └── lib.rs
    └── shops
        ├── Cargo.toml
        ├── sqlx.toml
        └── src
            └── lib.rs
```

## Disabling sub-crates with feature gates

Normal repository types would be replaced using feature gates and the same approach can be applied here, of course they would need to be applied in tandem with service, ports, repository and adapters etc - to cover the full "hot" path.

Axum route handlers can be gated to handle their disabled state. We can now run the same binary with different feature gates enabled. Imagine,

- container A: binary with feature "shop" enabled
- container B: binary with feature "reports" enabled

Both connect to the same (production) DB, yet each binary will only handle a single vertical.

```rs
use postgres_interfaces::PostgresShops;
#[cfg(feature = "shops")]
pub use postgres_shops;

#[cfg(feature = "shops")]
pub type PostgresShopsFacade = GenericPostgresShopsFacade<postgres_shops::ShopManager>;
#[cfg(not(feature = "shops"))]
pub type PostgresShopsFacade = GenericPostgresShopsFacade<DummyPostgresShopManager>;

/// Uses the Facade structural pattern
/// Ref: https://refactoring.guru/design-patterns/facade
pub struct GenericPostgresShopsFacade<S: PostgresShops> {
    pub manager: S,
}

impl<S: PostgresShops> GenericPostgresShopsFacade<S> {
    pub const fn new(manager: S) -> Self {
        Self { manager }
    }
}

pub struct DummyPostgresShopManager;
impl PostgresShops for DummyPostgresShopManager {}
```

## Potential Caveats

`sqlx::query!` macros _may_ only work in sub-crates, however, whilst writing an integration test (in an unrelated crate, meaning one that does not have a `sqlx.toml` file) seems to just work fine. Even running `cargo sqlx prepare --workspace -- --all-targets` picked up these changes in the integration tests.

## Conclusion

I also wrote about some [challenges I was facing with Postgres namespaces](https://www.reddit.com/r/rust/comments/1o3u9kr/simple_hexarch_crate_for_ledger_system_advice_pls/) and that discussion lead me in this direction.

What are your thoughts in adding another layer of abstraction, would you consider this to be extreme?

P.S. this is not limited to Axum, but I named the article as such as may extend this in the future with code examples of Axum handlers etc.

- [SQLx 0.9.0-alpha.1 released! `smol`/`async-global-executor` support, configuration with `sqlx.toml` files, lots of ergonomic improvements, and more!](https://www.reddit.com/r/rust/comments/1o72hab/sqlx_090alpha1_released_smolasyncglobalexecutor/)
- [Reddit thread](https://www.reddit.com/r/rust/comments/1oarzsm/axum_multitenancy_with_hexarch_and_abstracting/) for this article.
