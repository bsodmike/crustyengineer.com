+++
title = "Dark Arts of Unsafe Rust Programming"

[taxonomies]
tags = [ "rust", "unsafe", "undefined behaviour"] 
+++

This is not a typical blog post -- at least, not just yet. Right now, consider this a space of "clippings", examples which are to be expanded over time.

## Example 1: Danger of `*mut T`: Violating Rust’s type and aliasing invariants (as an example of UB)

> Raw pointers in Rust really just try to let you do whatever, but they have exactly one safety feature: because most people have no idea that variance and subtyping are a thing in Rust, and being incorrectly covariant would be horribly dangerous, `*mut T` is invariant, because there's a good chance it's being used "as" `&mut T`.

-- [Learning Rust With Entirely Too Many Linked Lists: A Production Unsafe Deque > Variance and PhantomData][1]

Here's a contrived example of how this can _go wrong_; as such my example is intentionally UB (Undefined Behaviour).

```rs
fn main() {
    // Convert mutable reference to a raw ptr
    let mut x: u32 = u32::MAX;
    let raw_x: *mut u32 = &mut x;

    // UNSAFE: pretend this raw pointer is a pointer to i32
    let raw_y: *mut i32 = raw_x as *mut i32;
    // without the cast above, we would see this error.
    //
    //   |     let raw_y: *mut i32 = raw_x;
    //   |                --------   ^^^^^ expected `*mut i32`, found `*mut u32`
    //   |                |
    //   |                expected due to this
    //   |
    //   = note: expected raw pointer `*mut i32`
    //              found raw pointer `*mut u32`

    unsafe {
        // Convert the raw pointer to a mutable reference of the wrong type
        // `*raw_y`: deref the ptr to the memory it points to and take a mutable reference
        // to that memory as type `&mut i32`
        let mut_ref: &mut i32 = &mut *raw_y;
        assert_eq!(x, u32::MAX);

        // UB: i32's minimum value is `-2147483648`, however, since we have intentionally
        // corrupted this, the value persisted looses its "sign" information.
        *mut_ref = i32::MIN; // writing as if it's i32

        assert_eq!(x, 2147483648u32);
    }
}
```

Breaking it down:

- `x` is created as a `u32`
- take a mutable reference to `x`, and convert it to a raw pointer `*mut u32` named `raw_x`.
- See the `UNSAFE` comment below, as we have to specify re-cast our raw pointer to `*mut i32`. This is the _dangerous_ part. This is the kind of UB one wants to avoid -- why?
- Refer to the unsafe block at the end. Recall that we initialised `x` as the maximum value of u32 - I did this intentionally, as I want to show that we can completely erase the _meaning_ of our data by causing UB.
- Since our raw pointer is a `*mut i32`, we can persist `i32::MIN`, the smallest (negative) `i32` value.
- However, it is incorrectly backed by a `u32`, which has no clue or care about negation since its Most Significant Big (MSB) is not used for this purpose.
- We write `-2147483648` as `i32`, however, this is stored as `2147483648` `u32` -- The observed change is not a numeric conversion but a consequence of writing raw bytes through a mis-typed mutable reference.

This demonstrates type reinterpretation causing undefined behaviour (UB) due to violating Rust’s type and aliasing invariants. The article on [Learning Rust With Entirely Too Many Linked Lists][1] is highly recommended<sup>[1][1]</sup> as this has a longer example on how this danger can present itself in a collection.

> Things that are "like mutable references" are invariant which means they block subtyping from happening on their generic parameters. So for safety, `&mut T` is invariant over `T`, and `Cell<T>` is invariant over `T` because `&Cell<T>` is basically just `&mut T` (because of interior mutability).
>
> Almost everything that isn't invariant is covariant, and that just means that subtyping "passes through" it and continues to work normally (there are also contravariant types that make subtyping go backwards but they are really rare and no one likes them so I won't mention them again).
>
> Collections generally contain a mutable pointer to their data, so you might expect them to be invariant too, but in fact, they don't need to be! Because of Rust's ownership system, `Vec<T>` is semantically equivalent to `T`, and that means it's safe for it to be covariant!

## Example 2: A note on raw pointers and `unsafe` blocks.

For new comers, it may not be immediately obvious, but the type system does not track raw pointers (`*const T` and `*mut T`).

This particular example is from a paper ["Stacked Borrows: An Aliasing Model for Rust"][2]. As the paper explains it:

> `&mut *raw_pointer` dereferences the raw pointer and immediately takes the address again, so this is effectively a cast from `*mut i32` back to `&mut i32`. The sinister part about this cast is that we do it twice! The type system does not stop us, as it does not even attempt to track what happens with raw pointers. As a result of all of this, we call `example1` with two aliasing references that both point to `local`, and the function returns `13`.

-- ["Stacked Borrows: An Aliasing Model for Rust, Page 2"][2]

Of course, in safe Rust, this would have just been `let result = example1(&mut 0, &mut 0)`, returning `42`, as one would expect.

```rs
fn example1(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    *y = 13;

    // Normally, `x` has to read 42 , because x and y cannot alias!
    return *x;
}

fn main() {
    let mut local = 5;
    let raw_pointer = &mut local as *mut i32;
    let result = unsafe { example1(&mut *raw_pointer, &mut *raw_pointer) };
    //     ^
    //     |
    //     \___this prints 13

    println!(" {} ", result);
}
```

[1]: https://rust-unofficial.github.io/too-many-lists/sixth-variance.html "Learning Rust With Entirely Too Many Linked Lists: A Production Unsafe Deque > Variance and PhantomData"
[2]: https://plv.mpi-sws.org/rustbelt/stacked-borrows/ "Stacked Borrows: An Aliasing Model for Rust, Ralf Jung, Hoang-Hai Dang, Jeehoon Kang, Derek Dreyer. POPL 2020"
