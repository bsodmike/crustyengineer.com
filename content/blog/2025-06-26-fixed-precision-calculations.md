+++
title = "Diving into fixed-precision calculations with fastnum"
# description = ""


[taxonomies]
tags = [ "rust", "financial trading"] 
+++

If we look at [one classic example][1] we know that computers are designed around 1's and 0's, hence the binary system, or base-2. This falls apart when looking at numbers such as `1/3` which evaluates to _0.333333333333333333333333333333333333_… and so on until infinity. That’s right much like 1/10 in binary, 1/3 in decimal also does not have a finite representation and any attempt to store it in a computer using a decimal number will result in a loss of precision. Decimal can display more fractions precisely than binary can, but not all of them.

This is [why financial systems store the base value][6] in cents. However, computations over aggregate data invariably leads to fixed-point calculations.

## Semantic typing

We can use the `newtype` pattern (a.k.a [New Type Idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)) in Rust and introduce such a type into our codebase.

```rust
use fastnum::D128;

#[derive(Debug, Clone, Copy, Default)]
pub struct Amount<const DECIMALS: usize>(D128);

/// Semantic type to indicate the underlying value is in Euros and not [`Cents`].
type Euros = Amount<0>;

/// A monetary amount in cents (2 decimal places).
#[allow(dead_code)]
type Cents = Amount<2>;

impl<const DECIMALS: usize> Amount<DECIMALS> {
    pub const fn new_f64(inner: f64) -> Self {
        Self(D128::from_f64(inner))
    }
}

impl<const DECIMALS: usize> std::fmt::Display for Amount<DECIMALS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const DECIMALS: usize> From<D128> for Amount<DECIMALS> {
    fn from(value: D128) -> Self {
        Self(value)
    }
}
```

In this example, we are formatting an aggregate value. The error here shows the high-precision of our [`D128`](https://docs.rs/fastnum/0.2.10/fastnum/decimal/type.D128.html) type.

```rs
// https://github.com/bsodmike/rust-scratch/tree/master/fixed_precision_calculations/src/ex_1_part_1.rs

#[should_panic(expected = "assertion `left == right` failed")]
#[test]
fn simulate_rounding_failure() {
   let average: f64 = 56098.9;
   let r: D128 = D128::from(average) / D128::from(100);

   assert_eq!(r.to_string(), "560.99");

   // thread 'ex_1_part_1::tests::simulate_rounding_failure' panicked at src/ex_1_part_1.rs:44:9:
   //     assertion `left == right` failed
   // left: "560.98900000000001455191522836685180664"
   // right: "560.99"
}

#[test]
fn simulate_rounding_failure_fixed() {
   let average: f64 = 56098.9;
   let r: D128 = D128::from(average) / D128::from(100);
   let r = r.round(2);

   assert_eq!(r.to_string(), "560.99");
}
```

We can leverage our semantic typing to reflect that the underlying value are Euros, or `Amount<0>`. Rounding is performed since we only care about 2 decimal places.

```rs
// https://github.com/bsodmike/rust-scratch/tree/master/fixed_precision_calculations/src/ex_1_part_1.rs

#[should_panic(expected = "assertion `left == right` failed")]
#[test]
fn simulate_rounding_failure_converted() {
   let average: f64 = 56098.9;
   let r: D128 = D128::from(average) / D128::from(100);
   // let r = r.round(2);
   let euros: Euros = r.into();

   assert_eq!(euros.to_string(), "560.99");
   //
   // thread 'ex_1_part_1::tests::simulate_rounding_failure_converted' panicked at src/ex_1_part_1.rs:68:9:
   //     assertion `left == right` failed
   // left: "560.98900000000001455191522836685180664"
   // right: "560.99"
}

#[test]
fn simulate_rounding_failure_converted_fixed() {
   let average: f64 = 56098.9;

   let r: D128 = D128::from(average) / D128::from(100);
   let euros: Euros = r.round(2).into();

   assert_eq!(euros.to_string(), "560.99");
}
```

## The `fastnum` crate

> Any `fastnum` decimal type consists of an N-bit big unsigned integer, paired with a 64-bit control block which contains a 16-bit scaling factor determines the position of the decimal point, sign, special, and signaling flags. Trailing zeros are preserved and may be exposed when in string form.

I also found a great quote from the author of `fastnum`:

> the primary purpose of this library – to provide strictly accurate precision with no round-off errors, within the rules of the decimal number system. Naturally, it offers no particular advantage for general rational numbers. In fact, in any numeral system (e.g., base-2, base-10, or base-16), there will always be fractions that can't be represented with a finite number of digits.
>
> The key point is that working with decimal numbers follows intuitive rules familiar to everyone from school. For example, we all understand that 1/3 = 0.333333...(3) and that rounding is eventually inevitable. However, the fact that 0.1, when written down in a notebook, might turn into something like 0.10000000000001 in calculations – puzzles many people, because in the real world, we neither interact with the binary number system nor write numbers in it.

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
