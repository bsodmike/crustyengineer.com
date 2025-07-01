+++
title = "Semantic Types for Money in Rust, with Better Precision and Fixed-point Decimal Arithmetic"
# description = ""


[taxonomies]
tags = [ "rust", "financial trading"] 
+++

At the end of this article, I have added further reading material which cover the following topics:

- Why using floats for currencies is generally a bad idea. This is due to floats being a `base-2` (binary) number system, and will always lead to decimal rounding errors.
- Recommended approach is to store monetary values (currencies) as "cents" (or higher scaling as needed -- see examples for further details).

## Quick overview of the `fastnum` crate

Reasons as to why one would choose this particular crate:

- `fastnum` is a crate that implements fixed-precision calculations using fixed-point decimal arithmetic, which could be summarised succinctly as `coefficient / 10^exponent = 12345 / 100 = 123.45` for a `coefficient` of `12345` and `exponent` of `2`.
- the `fixed-precision` aspect of this crate makes it blazing fast, when compared to alternatives.
- The [codebase is only 8-months (new)](https://docs.rs/fastnum/0.2.10/fastnum/index.html) as of the time of typing and appears to be popular and well maintained on Github.
- Disclaimer: this is now a core dependency of a financial platform that I'm working on for a client. The codebase uses semantic types, and my work largely interacts with them. This article is inspired by this particular implementation.

From the author of `fastnum`:

> The key point is that working with decimal numbers follows intuitive rules familiar to everyone from school. For example, we all understand that 1/3 = 0.333333...(3) and that rounding is eventually inevitable. However, the fact that 0.1, when written down in a notebook, might turn into something like 0.10000000000001 in calculations – puzzles many people, because in the real world, we neither interact with the binary number system nor write numbers in it.

He refers to the [classic example of this is 0.1 + 0.2 ≠ 0.3][1]. Since the binary number system, `base-2` is the building block of logic, this falls apart when looking at numbers such as `1/3` which evaluates to _0.33333333333333_… and so on until infinity. That’s right much like 1/10 in binary, 1/3 in decimal also does not have a finite representation and any attempt to store it in a computer using a decimal number will result in a loss of precision. Decimal can display more fractions precisely than binary can, but not all of them.

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

#[derive(Debug, Clone, Copy, Default)]
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

- `new_scaled_i32` converts an `i32` value returning our `Amount<D>` type. From the example below, this is `1234/(10^2) = 12.34`
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

We can also format this value to a `String`, either directly

```rust
    #[test]
    fn convert_to_string() {
        // This is a whole currency unit
        let value = Amount::<0>::new_scaled_i32(1234);
        let formatted = format!("{}", value);

        assert_eq!(formatted, "1234");

        let value = Amount::<2>::new_scaled_i32(1234);
        let formatted = format!("{}", value);

        assert_eq!(formatted, "12.34");

        // We've increased our precision here, this is reflected in the formatted output
        let value = Amount::<4>::new_scaled_i32(123456);
        let formatted = format!("{}", value);

        assert_eq!(formatted, "12.3456");
    }
```

or preferably via the types we introduced earlier. `Pertenthousand` is useful in financial systems, as seen in High-frequency Trading (HFT) platforms; notice that we no longer need to provide annotation to the compiler for the generic type `D` in `Amount<D>` as this is already specified via our semantic types.

```rust
    #[test]
    fn convert_to_string_via_semantic_types() {
        // This is a whole currency unit
        let value: Euros = Amount::new_scaled_i32(1234);
        let formatted = format!("{}", value);
        assert_eq!(formatted, "1234");

        // monetary cents
        let value: Cents = Amount::new_scaled_i32(1234);
        let formatted = format!("{}", value);
        assert_eq!(formatted, "12.34");

        // We've increased our precision here, this is reflected in the formatted output
        let value: Pertenthousand = Amount::new_scaled_i32(123456);
        let formatted = format!("{}", value);
        assert_eq!(formatted, "12.3456");
    }
```

We need to impl the `Display` trait for this to work

```rust
impl<const DECIMALS: usize> std::fmt::Display for Amount<DECIMALS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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
