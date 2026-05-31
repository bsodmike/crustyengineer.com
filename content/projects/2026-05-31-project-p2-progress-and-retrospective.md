+++
title = "Implementing Double-Entry Bookkeeping with Postgres & Rust — Retrospective"
toc = true

[extra]
toc = true

[taxonomies]
tags = [ "rust"] 
+++

## Overview

For the longest time I have managed my expense tracking using spreadsheets, and this seems to be the approach others usually take. It's quick, easy, and gets the job done without too much hassle. It's organic, and usually something we pickup early on and over the years it becomes a creature of habit.

Couple years back though, I found myself performing annual "reconsolidation" of investments spanning multiple banks and also multiple currencies. It wasn't long before I became obsessed in learning more and more about ledgers.

I was also keen on leveraging "Double-entry Bookkeeping" (DBE), also referred to the shock and horror of some as "Double-entry Accounting" (DBA) &mdash; which sadly is easy to confused with Database administration!

Things start to get dicey pretty quickly once you step into the world of accounting, as terminology start to overlap between the accounting, business, and domains.

### Plain text accounting options

We have the "what", but before we can ask the "how", let's also consider the "why?". This required a wee bit of research and I _always_ reach out for open source solutions first, and this lead me to

- [ledger-cli.org](https://ledger-cli.org/)
- [gnuCash](https://gnucash.org/)

Since my existing workflow is spreadsheet based, using CSV data as an input via a CLI command seemed like the next obvious step. The CSV columns will require parsing and this allows us to ensure a strict standard on the quality of input data.

I have witnessed first hand how much havoc "bad" data can have inside a production system. Clean parsed, validated, and sanitized data is far easier to manage than the alternative.

## Stack & High level design

At the outset the stack I chose to use was

- Multi-tenant design
- Persistence: Postgres & `sqlx`
- CLI with [clap](https://crates.io/crates/clap)
- PDF output with [typst](https://crates.io/crates/typst)
- HTTP server with [axum](https://crates.io/crates/axum)

Development was planned in sprints to address

1. CLI import of `accounts.json` and `transfers.csv` and display an overall computation into STDOUT.
2. Export PDF via custom templates using `Typst`.
3. HTTP server + JWT authentication + "Clean architecture" between HTTP handlers and DB layers.

## Low-level design

During my [prior work on multi-tenancy](https://crustyengineer.com/blog/axum-multi-tenancy-abstract-repository-layer/) with `sqlx`, I also observed it had multi-database support, which was far more suitable.

The following crates are used for the purpose of migrations, and revolve around 3 databases.

1. PRE_FIX_main: this is the top-level `sqlx` crate. This has a src/migrations dir.
2. PRE_FIX_auth
3. PRE_FIX_ledger

```shell
# DB: PRE_FIX_main
./http_api/
├── Cargo.toml
├── sqlx.toml
└── src

./auth & ./ledger
├── Cargo.toml
├── migrations
├── sqlx.toml
└── src
```

## Progress thus far

## "Seeing the wood for the trees"

This is an idiom that means _understanding the overall situation without letting small, specific details cloud your perspective_.

This project involves aspects that required my learning of double-entry booking keeping & accounting, wading to layers of "accounting" jargon and then distilling it down to the most trivial "primitive" whilst attempting a complex amalgamation of technologies. I was left with a lot more questions at the start, and a lot of the specific details lead to a lot of uncertainty.

- "How do I implement X"
- "How do I implement it correctly, whilst keeping it simple yet maintainable?"

## Progress

## Overview of progress

To address how development has progressed, my initial flow was driving mainly by "how..." & "what's the best way to do x..."

What's the cleanest, simplest, and scalable way to represent a ledger?

## Alchemy of technologies

The current solution is a blend of a few "technologies"

- Postgres migrations with `sqlx` in a multi-database capacity, to the point of keeping tenant (JWT) data in a separate auth DB, and a separate DB for everything related to the ledger tables.
- Rust APIs separating `ledger`, `ledger_cli`, and `http_api` (Axum + Clean architecture)
- Clean architecture is a blend of DDD & Hex Arch, leaning more towards Hexagonal architecture.
- `ledger_cli` package with Clap

# Breakthrough #1: CLI first & starting very small.

By February 2026 I had a horrible "paperwork" (p1) build, but it was extremely messy and I had lost vision of its boundaries. The first correction I did was to start in a new clean build with cargo and start with just two crates (1) `ledger` and (2) `ledger_cli` and nothing else.

The output of this project is the CLI output you see above; this was then extended with a reporting module and integrating `Typst` to generate various PDF, ultimately till I settled on a "Composite Balances Sheet". This was all done in away where DB facing calls were completely isolated through its own LedgerManager struct.

Once you maintain clear isolated boundaries, it is much easier and cleaner to start composing on top.

```

```
