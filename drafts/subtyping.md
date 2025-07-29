# Subtyping and Variance

## Subtyping

The [Rustonomicon](https://doc.rust-lang.org/nomicon/subtyping.html#subtyping) states that

> Subtyping is the idea that one type can be used in place of another.
>
> Let's define that `Sub` is a subtype of `Super` (we'll be using the notation `Sub <: Super` throughout this chapter).
>
> What this is suggesting to us is that the set of _requirements_ that `Super` defines
> are completely satisfied by `Sub`. `Sub` may then have more requirements.
>
> Now, in order to use subtyping with lifetimes, we need to define the requirement of a lifetime:
>
> > `'a` defines a region of code.
>
> Now that we have a defined set of requirements for lifetimes, we can define how they relate to each other:
>
> > `'long <: 'short` if and only if `'long` defines a region of code that **completely contains** `'short`.
>
> `'long` may define a region larger than `'short`, but that still fits our definition.

This can be understood better as,

> "Sub is a subtype of Super" means that "every Sub is a Super" and "any operation that takes a Super argument can be also given a Sub argument and still work". Alternatively, "Sub can be converted into Super"

The intended order for reading this is that `Sub` is below `Super`. You can also read `'a: 'b` as "`'a` outlives `'b`".

Consider from the nomicon

> If we remember from the above examples, it was ok for us to treat &'a T as a subtype of &'b T if 'a <: 'b, therefore we can say that &'a T is covariant over 'a.
