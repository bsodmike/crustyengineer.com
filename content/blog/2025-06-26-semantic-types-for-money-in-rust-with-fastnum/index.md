+++
title = "Semantic Types for Money in Rust, with Better Precision and Fixed-point Decimal Arithmetic"
toc = true

[extra]
toc = true

[taxonomies]
tags = [ "rust", "financial trading"] 
+++

## Overview

> 💡 This section has been combined with an excellent overview of the subject provided by [WuBingzheng](https://github.com/WuBingzheng) from his article [Comparison and Benchmarking of Rust Decimal Crates](https://wubingzheng.github.io/en/Decimal-Crates-Comparison.html#benchmark-comparison).
>
> His intro material is targeted at new audiences, by his own admission, and has been added to improve context in my article. I do not see the need to "reinvent" the wheel, when he has already provided excellent coverage and depth on the topic.
>
> I also share his sentiment to avoid ambiguity and use Fixed-size instead of Fixed-precision, where applicable.

Since the binary number system, `base-2` is the building block of logic, this falls apart when looking at numbers such as `1/3` which evaluates to _0.33333333333333_… and so on until infinity. Binary fractions cannot represent decimal fractions exactly. Consider for example, `f64` as this is a good example of this [classic arithmetic error: `0.1 + 0.2 ≠ 0.3`][1]. Here's a quick quote from the author of `fastnum`:

> The key point is that working with decimal numbers follows intuitive rules familiar to everyone from school. For example, we all understand that 1/3 = 0.333333...(3) and that rounding is eventually inevitable. However, the fact that 0.1, when written down in a notebook, might turn into something like 0.10000000000001 in calculations – puzzles many people, because in the real world, we neither interact with the binary number system nor write numbers in it.

That’s right much like 1/10 in binary, 1/3 in decimal also does not have a finite representation and any attempt to store it in a computer using a decimal number will result in a loss of precision. Decimal can display more fractions precisely than binary can, but not all of them.

In financial applications we require exact representation of decimal fractions. This is why decimal crates are needed. These use integers to represent the mantissa, along with a scale representing the number of decimal places. For example, the value `1.23` can be represented using integer `123` with `scale = 2`.

There are many decimal crates in the Rust ecosystem, each with different designs
and trade-offs. Their differences mainly fall into two dimensions:

1. Whether the scale is fixed or variable. This corresponds to
   [Fixed-point](https://en.wikipedia.org/wiki/Fixed-point_arithmetic)
   vs [Floating-point](https://en.wikipedia.org/wiki/Floating-point_arithmetic).

2. Whether the count of integers are fixed or arbitrary. This corresponds to
   [Fixed-precision](https://en.wikipedia.org/wiki/Fixed-precision_arithmetic)
   vs [Arbitrary-precision](https://en.wikipedia.org/wiki/Arbitrary-precision_arithmetic).

The first two sections ([Fixed-point and Floating-point](#fixed-point-and-floating-point),
[Fixed-size and Arbitrary-precision](#fixed-size-and-arbitrary-precision))
introduce the characteristics of these categories. There is nothing
particularly new here, so experienced readers may skip them.

## Fixed-point and Floating-point

_[Fixed-point](https://en.wikipedia.org/wiki/Fixed-point_arithmetic)_ vs _[Floating-point](https://en.wikipedia.org/wiki/Floating-point_arithmetic)_.

In fixed-point arithmetic, the scale is fixed and bound to the type. In floating-point arithmetic, the scale is variable and stored in each instance.

Let's illustrate this with code. A typical _fixed-point_ type definition might look like this:

```rust
struct FixedPoint<const SCALE: i32>(i128); // scale is bound to type
```

A typical _floating-point_ decimal type might look like this:

```rust
struct FloatingPoint {
    mantissa: i128,
    scale: i32, // scale is stored in each instance
}
```

This clearly shows that fixed-point numbers have fixed decimal precision, while floating-point decimals have variable precision. For example, `FixedPoint<2>` always has 2 decimal places, while the precision of `FloatingPoint` depends
on each instance's scale.

Because of this distinction, _fixed-point_ and _floating-point_ types exhibit the following differences:

1. Fixed-point numbers have a smaller representable range, while floating-point
   numbers can represent a much larger range. This is because floating-point
   numbers sacrifice decimal precision as values become larger.

2. Fixed-point arithmetic is simpler and faster, while floating-point arithmetic
   is more complex and slower. For example, addition for fixed-point numbers only
   requires integer addition on the mantissa. Floating-point addition must first
   check whether the scales are equal (this check itself can already be slower
   than the addition), and if not, align the scales through multiplication. Refer to the [detailed discussed in the benchmark section](https://wubingzheng.github.io/en/Decimal-Crates-Comparison.html#benchmark-comparison).

3. Fixed-point arithmetic is somewhat more cumbersome to use, while floating-point
   arithmetic is more convenient. For example, with the `FixedPoint` type above,
   the scale must be determined at compile time for each type, such as how many
   decimal places `Balance` or `Price` should have. Floating-point decimals do
   not require this consideration.

The difference between the two is somewhat analogous to the difference between
statically typed and dynamically typed languages.

Most applications use decimal crates simply to represent decimal fractions exactly,
without particularly high requirements for performance or strict decimal precision.
In such cases, floating-point decimals are usually preferred for convenience.
However, for more serious services, especially many financial systems that require
strict decimal precision or high performance, fixed-point decimals are recommended.
For example, USD assets should have exactly 2 decimal places, neither more nor less.

NOTE: Since built-in floating-point types in programming languages (such as C's
`float` and `double`, or Rust's `f32` and `f64`) are commonly referred to as
"floating-point", and these types cannot represent decimal fractions exactly,
many people mistakenly think that "floating-point" inherently cannot represent
decimal fractions exactly. This is WRONG! More precisely, these are "binary
floating-point" numbers. The inability to represent decimal fractions exactly
comes from the "binary" part, not the "floating-point" part. Because people
often omit the word "binary", floating-point arithmetic unfairly gets blamed.
In fact, even _binary fixed-point_ types, such as the
[`fixed`](https://docs.rs/fixed/latest/fixed/) crate, also cannot represent
decimal fractions exactly. As long as a crate is decimal-based, whether
fixed-point or floating-point, it can represent decimal fractions exactly.

NOTE: Floating-point arithmetic has a standard called
[IEEE 754](https://en.wikipedia.org/wiki/IEEE_754), which defines both binary
floating-point formats (used by `f32`/`f64`) and decimal floating-point formats.
However, this standard is only _one_ implementation approach for floating-point
arithmetic, not the entirety of floating-point arithmetic itself. Other
implementations are also possible. In practice, most decimal crates do not
follow IEEE 754 decimal formats.

## Fixed-size and Arbitrary-precision

_[Fixed-precision](https://en.wikipedia.org/wiki/Fixed-precision_arithmetic)_ vs
_[Arbitrary-precision](https://en.wikipedia.org/wiki/Arbitrary-precision_arithmetic)_.

First, let's clarify the meaning of the word "precision" here. The term has two conflicting meanings:

- Number of fraction places
- Number of significant digits

For example, the value `1.23` has 2 fraction places but 3 significant digits.
Both meanings are widely used. For example,
[std::fmt](https://doc.rust-lang.org/std/fmt/index.html#precision) uses the
former meaning, while here (Fixed-precision vs Arbitrary-precision) the latter
meaning is used. This is the [standard terminology](https://en.wikipedia.org/wiki/Fixed-precision_arithmetic),
but it easily causes confusion. "Fixed-precision" is often misunderstood as
fixed fraction places, leading to confusion with fixed-point arithmetic.

To avoid ambiguity, this article uses the term _Fixed-size_ instead of _Fixed-precision_.

As the name suggests, Fixed-size types use a fixed number of integers (one or more).
Arbitrary-precision types use as many integers as necessary: expanding to the
left to avoid overflow, and expanding to the right to avoid precision loss.

Naturally, this requires heap allocation, meaning the type is not `Copy`,
and the crate is not `no-alloc`. All operations also become significantly slower.
Unless there is a clear requirement for arbitrary precision, Fixed-size types
are generally preferable.

## Choosing Crates & Benchmarks

Refer to [Comparison and Benchmarking of Rust Decimal Crates: Benchmarking](https://wubingzheng.github.io/en/Decimal-Crates-Comparison.html#choosing-crates) by [WuBingzheng](https://github.com/WuBingzheng).

The most common and popular are `bigdecimal` (Floating-point / Arbitrary-precision), `fastnum`, and `rust_decimal`, where both are Floating-point & Fixed-size.

## Quick overview of the `fastnum` crate

Reasons as to why one would choose this particular crate:

- `fastnum` is a crate that implements fixed-precision calculations using fixed-point decimal arithmetic, which could be summarised succinctly as `coefficient / 10^exponent = 12345 / 100 = 123.45` for a `coefficient` of `12345` and `exponent` of `2`.
- The [codebase is only 8-months (new)](https://docs.rs/fastnum/0.2.10/fastnum/index.html) as of the time of typing and appears to be popular and well maintained on Github.
- Disclaimer: this is now a core dependency of a financial platform that I'm working on for a client. The codebase uses semantic types, and my work largely interacts with them. This article is inspired by this particular implementation.

Here's another great write up, explaining why [financial systems store the base value][6] in cents:

> Because floats and doubles cannot accurately represent the base 10 multiples that we use for money. This issue isn't just for Java, it's for any programming language that uses base 2 floating-point types.
>
> In base 10, you can write 10.25 as 1025 _ 10-2 (an integer times a power of 10). IEEE-754 floating-point numbers are different, but a very simple way to think about them is to multiply by a power of two instead. For instance, you could be looking at 164 _ 2-4 (an integer times a power of two), which is also equal to 10.25. That's not how the numbers are represented in memory, but the math implications are the same.
>
> Even in base 10, this notation cannot accurately represent most simple fractions. For instance, you can't represent 1/3: the decimal representation is repeating (0.3333...), so there is no finite integer that you can multiply by a power of 10 to get 1/3. You could settle on a long sequence of 3's and a small exponent, like 333333333 \* 10-10, but it is not accurate: if you multiply that by 3, you won't get 1.
>
> However, for the purpose of counting money, at least for countries whose money is valued within an order of magnitude of the US dollar, usually all you need is to be able to store multiples of 10-2, so it doesn't really matter that 1/3 can't be represented.
>
> The problem with floats and doubles is that the vast majority of money-like numbers don't have an exact representation as an integer times a power of 2. In fact, the only multiples of 0.01 between 0 and 1 (which are significant when dealing with money because they're integer cents) that can be represented exactly as an IEEE-754 binary floating-point number are 0, 0.25, 0.5, 0.75 and 1. All the others are off by a small amount. As an analogy to the 0.333333 example, if you take the floating-point value for 0.01 and you multiply it by 10, you won't get 0.1. Instead you will get something like 0.099999999786...
>
> Representing money as a double or float will probably look good at first as the software rounds off the tiny errors, but as you perform more additions, subtractions, multiplications and divisions on inexact numbers, errors will compound and you'll end up with values that are visibly not accurate. This makes floats and doubles inadequate for dealing with money, where perfect accuracy for multiples of base 10 powers is required.
>
> A solution that works in just about any language is to use integers instead, and count cents. For instance, 1025 would be $10.25. Several languages also have built-in types to deal with money

-- Source: [https://stackoverflow.com/a/3730040](https://stackoverflow.com/a/3730040)

Let's cover our first topic.

## Semantic typing

We can use the `newtype` pattern (a.k.a [New Type Idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)) in Rust and introduce such a type into our codebase. It will be backed by [`D128`](https://docs.rs/fastnum/0.2.10/fastnum/decimal/type.D128.html) from the `fastnum` crate.

```rust
use fastnum::D128;

// This example has the entire kitchen sink!
#[derive(Debug, Clone, Copy, Eq, Ord, PartialOrd, PartialEq, Hash, Default)]
pub struct Amount<const DECIMALS: usize>(D128);

/// Semantic type to indicate the underlying value is in Euros and not [`Cents`].
type Euros = Amount<0>;

/// A monetary amount in cents (2 decimal places).
#[allow(dead_code)]
type Cents = Amount<2>;

/// A monetary amount in cents/100 (4 decimal places), or "1/10,000" - hence the name.
pub type Pertenthousand = Amount<4>;
```

Let's expand the interface for our `Amount<D>` type, starting with two methods.

- `new_scaled_i32`: when creating a value of `Amount<D>`, we use the internal storage to scale it by `N`. In the context of cents `N = 2` and thus, `new_scaled_i32(1234)` is stored internally as `D128(digits=[1234], exp=[-2]`.

- `new_f64` creates a [`D128`](https://docs.rs/fastnum/0.2.10/fastnum/decimal/type.D128.html) value, just as it says on the tin.

```rust
impl<const DECIMALS: usize> Amount<DECIMALS> {
    /// Treats the input as a scaled integer (e.g. 1234 → 12.34)
    pub const fn new_scaled_i32(inner: i32) -> Self {
        Self(D128::from_i32(inner).div(D128::from_i32(10_i32).pow(D128::from_usize(DECIMALS))))
    }

    pub const fn new_f64(inner: f64) -> Self {
        Self(D128::from_f64(inner))
    }
}
```

Consider the following example:

```rs
    #[test]
    fn convert_from_f64_using_new() {
        // Assume we have a whole currency unit, parsed from a CSV file, into a f64.  Since this is
        // a whole currency unit, we need to convert it to cents before we can use our semantic type.
        let provided = 1.23;
        // Notice in `new_scaled_i32` above this is scaled before storage, and we can see it is internally
        // stored with a scaling factor of N
        let converted: Cents = Amount::new_scaled_i32((provided * 100.00) as i32);
        assert!(format!("{:?}", &converted).contains("D128(digits=[123], exp=[-2]"));

        // converting from internal storage, this is scaled by N of Amount<N>
        let d: i32 = converted.into();
        assert_eq!(d, 123_i32);

        // Let's do the same but change our original value; assume we have 1 Euro:
        let provided = 1.00;
        let converted: Cents = Amount::new_scaled_i32((provided * 100.00) as i32);
        // Interestingly, this is stored as 1e0 (which is the same as 100e-2).
        assert!(format!("{:?}", &converted).contains("D128(digits=[1], exp=[0]"));

        // converting from internal storage, this is scaled by N of Amount<N>
        let d: i32 = converted.into();
        assert_eq!(d, 100_i32);
    }
```

## Example: Parsing values from CSV

Assume we have a CSV with column data, that we've sufficiently cleaned, let's parse this into `Cents`. In the example below we are handling Euros, which commonly uses `,` as the decimal separator.

- Replace `,` to `.`
- Parse `&str` values as `f64`
- As per our previous example, since the source values are Euros, we need to first convert to cents -- otherwise our semantic typing will be wrong.

```rs
/// Parse amount (Euros) from CSV file into Cents.
fn parse_amount(value: &str) -> Result<Cents, CsvParseError> {
    let value = value.trim();

    if value.is_empty() {
        return Err(CsvParseError::AmountMissingError(String::default()));
    }

    // Parse amount converting from Euros to cents.
    let amount = (value.replace(',', ".").parse::<f64>().map_err(|err| {
        CsvParseError::AmountParseError(format!("Invalid amount: {value}: {err}"))
    })? * 100.00) as i32;

    Ok(amount.into())
}
```

## Github

Code examples for this article:

- [github.com/bsodmike/rust-scratch/tree/master/fixed_precision_calculations](https://github.com/bsodmike/rust-scratch/tree/master/fixed_precision_calculations).

## Further reading

### 📉 Binary Conversion & Precision Loss (General)

1. [**The Bad, the Good and the Ugly of Binary Floating Point Numbers**][1]  
   Explains why numbers like `0.1 + 0.2 != 0.3` occur and demystifies binary fractional representations.

2. [**Maybe Demystifying Floating Point Precision**][2]  
   In-depth explanation of mantissa/exponent structure and why decimal digits are lost.

3. [**Binary Representation of Floating-Point Numbers**][3]  
   Interactive visual tool showing why some decimal values cannot be exactly represented in binary.

4. [**What Every Computer Scientist Should Know About Floating-Point**][4]  
   Classic paper explaining IEEE‑754 floating-point math, edge cases, and precision traps.

5. [**Solving the “Floating Point Precision” Problem With... Floats?**][5]  
   Explores workarounds using floats with fixed-precision scaling techniques.

### 💰 Best Practices in Financial Calculations (General)

1. [**Precision Matters: Why Using Cents Instead of Floating Point Is Crucial**][6]  
   Demonstrates rounding bugs in real-world fintech apps and proposes storing cents as integers.

2. [**You Better Work in Cents, Not Dollars**][7]  
   Advice from fintech practitioners on using minor units to avoid floating-point inaccuracies.

3. [**Storing Currency Values: Data Types and Best Practices**][8]  
   Explains and compares floats, `Decimal` types, and integer cents with pros and cons.

4. [**Why Not Use Double or Float to Represent Currency?**][9]  
   High-voted StackOverflow answer with clear examples of float errors in currency math.

5. [**Handling Monetary Amounts**][10]  
   Overview of various ways to handle money in code, and their pitfalls.

6. [**Deciding on Money Data Types (Stripe Engineering)**][11]  
   How Stripe models money safely and why they avoid floats entirely.

### 🦀 Rust-Specific Articles & Discussions

1. [**Demystifying Floats in Rust: Precision, Performance, and When to Use Which**][12]  
   Compares `f32`, `f64`, and alternatives like fixed-point in Rust.

2. [**Handling Floating Point Challenges with Rust (Conf42 Talk)**][13]  
   Covers float comparison strategies, summation algorithms, and avoiding drift in Rust.

3. [**Idiomatic Way to Handle Floating Point Precision (Reddit)**][14]  
   Community suggestions on using crates like `rust_decimal`, `fastnum`, or `bigdecimal`.

4. [**Rust Users Forum: Floating Point Number Tricks**][15]  
   Tips and techniques for numerical accuracy and debugging float behavior.

5. [**Rust Users Forum: What’s this precision in float?**][16]  
   FAQ-style discussion on the decimal precision of `f32` (7 digits) and `f64` (15–16 digits).

[1]: https://medium.com/@jlabath/the-bad-the-good-and-the-ugly-of-binary-floating-point-numbers-5bee8f693ebc "The Bad, the Good and the Ugly of Binary Floating Point Numbers"
[2]: https://blog.demofox.org/2017/07/31/maybe-demystifying-floating-point-precision "Maybe Demystifying Floating Point Precision"
[3]: https://trekhleb.dev/blog/2021/binary-representation-of-floating-point-numbers "Binary Representation of Floating-Point Numbers"
[4]: https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html "What Every Computer Scientist Should Know About Floating-Point"
[5]: https://dev.to/voliva/solving-the-floating-point-precision-problem-with-floats-4369 "Solving the “Floating Point Precision” Problem With... Floats?"
[6]: https://www.pullrequest.com/blog/precision-matters-why-using-cents-instead-of-floating-point-for-transaction-amounts-is-crucial "Precision Matters: Why Using Cents Instead of Floating Point Is Crucial"
[7]: https://blog.agentrisk.com/you-better-work-in-cents-not-dollars-f41884f7ae0b "You Better Work in Cents, Not Dollars"
[8]: https://cardinalby.github.io/blog/posts/storing-currency-values-data-types-best-practices "Storing Currency Values: Data Types and Best Practices"
[9]: https://stackoverflow.com/questions/3730019/why-not-use-double-or-float-to-represent-currency "Why Not Use Double or Float to Represent Currency?"
[10]: https://medium.com/@leon.klinkea/handling-monetary-amounts-7393e0d77ac7 "Handling Monetary Amounts"
[11]: https://stripe.com/blog/money-data-types "Deciding on Money Data Types (Stripe Engineering)"
[12]: https://medium.com/@murataslan1/demystifying-floats-in-rust-precision-performance-and-when-to-use-which-9a3ced352288 "Demystifying Floats in Rust: Precision, Performance, and When to Use Which"
[13]: https://www.conf42.com/Rustlang_2023_Prabhat_handling_floating_point_challenges "Handling Floating Point Challenges with Rust (Conf42 Talk)"
[14]: https://www.reddit.com/r/learnrust/comments/z5745e/idiomatic_way_to_handle_floating_point_precision "Idiomatic Way to Handle Floating Point Precision (Reddit)"
[15]: https://users.rust-lang.org/t/floating-point-number-tricks/52048 "Rust Users Forum: Floating Point Number Tricks"
[16]: https://users.rust-lang.org/t/whats-this-precision-in-float/102430 "Rust Users Forum: What’s this precision in float?"
