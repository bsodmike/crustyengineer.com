+++
title = "Rust: A Pragmatic Approach to Subtyping and Variance"
# description = ""


[taxonomies]
tags = [ "rust", "language foundations"] 
+++

The [reference material for these topics are the Rustonomicon][1], and most in the community will point new comers in this direction, but in honestly, one tends to just scratch the surface - the "Crust of Rust", if you will. The Rustonomicon will most often leave you with more questions than answers, and I am hoping to address a few of those here.

Rather aptly named, the popular YouTube series by [Jon Gjengset](https://www.youtube.com/@jonhoo) will be [used to aid in the material](https://www.youtube.com/watch?v=iVYWDIW71jk) covered here. I will be referencing some of this examples, but please refer to the section at the end for further resources.

## Variance, PhantomData, and drop check

Consider this first example,

```rs
    #[test]
    fn ex1_no_problemo() {
        let x = String::new();
        let z = vec![&x];

        // x is never used beyond this point, so it's ok to drop the vector. It checks every T (in Vec<T>) and run the Drop impl on each T.
        drop(x);
    }
```

If we try to drop the vector, which is holding references to `x` (which, has now been dropped) we will see the borrow check in action. `z` is still pointing to references of `x`, therefore we are not allowed to drop `z`.

```rs
let x = String::new();
let z = vec![&x];

drop(x);
// we cannot drop z as it is holding references to x.
drop(z); // <--- this will not compile

// error[E0505]: cannot move out of `x` because it is borrowed
// --> src/language_foundations/subtyping_variance_jon_gjengset.rs:8:14
// |
// 6 |         let x = String::new();
// |             - binding `x` declared here
// 7 |         let z = vec![&x];
// |                      -- borrow of `x` occurs here
// 8 |         drop(x);
// |              ^ move out of `x` occurs here
// 9 |         drop(z);
// |              - borrow later used here
```

You'll notice our Drop code for `TouchDrop` refers to `self.0`, which causes a move. What if we did `println!("Touch drop! {:?}", &self.0);`? Quite right, it compiles just fine.

```rs
    struct TouchDrop<T: Debug>(T);

    impl<T: Debug> Drop for TouchDrop<T> {
        fn drop(&mut self) {
            println!("Touch drop! {:?}", self.0);
        }
    }

    #[test]
    fn ex2_newtype_drop_impl_rustc_error() {
        let x = String::new();

        // this is commented out to allow tests to compile.
        // TouchDrop borrows a reference to x here.
        let z = vec![TouchDrop(&x)];

        // we cannot drop x, as it is borrowed above.
        drop(x);  // <--- this will not compile

        // we also have an implicit drop here, as it's the end of the block.  The compiler shows us
        // > borrow might be used here, when `z` is dropped and runs the `Drop` code for type `Vec`
        // DO NOT UNCOMMENT: this is only used for explanation purposes.
        // drop(z);
    }
```

The above examples are exactly the material that [Jon Gjengset](https://www.youtube.com/@jonhoo) presents in his video, and is important as we move onto the concept of `PhantomData`, and an interesting nuance around ownership, variance and the drop check.

### Using PhantomData to communicate intent to the Rust compiler

In this example our type `F<T>` (in this case `Deserializer<T>`) is described by [Jon Gjengset](https://www.youtube.com/@jonhoo) succinctly -- in terms on how the compiler decides on the Drop implementation of its inner type.

> The way the compiler knows this is, it looks at the generic parameters and it sees whether the type contains a `T` and a `PhantomData<T>` contains a `T` as far as the type system is concerned, it's considered as dropping a `T`.

```rs
    struct Deserializer<T> {
        // some fields
        _t: PhantomData<T>,
    }
```

However, with this signature `PhantomData<fn() -> T>`, it does not own a `T`, so it cannot drop a `T`. If you drop one of these, you're just dropping a function definition. That's why `PhantomData<fn() -> T>` is a preference compared to `PhantomData<T>` if you know that your type doesn't drop a `T`, such as a deserializer.

```rs
    struct Deserializer2<T> {
        // some fields
        // this is covariant, which will support shortening lifetimes.
        _t: PhantomData<fn() -> T>,
    }
    struct Deserializer3<T> {
        // some fields
        // this is contravariant, and would be really annoying to use as you would not be able to shorten lifetimes, say from `'static` to some `'a` (technically, from long to short, as it implies, but for the purpose of this article, my aim is to be explicit!).
        _t: PhantomData<fn(T)>,
    }
```

If you want your type to be invariant in `T` where you might have some interior mutability and so you need to have invariant so that people can't pull the same hack as we looked at for mutable references, you an do this. This is because it's trying to be both covariant and contravariant in `T`, which no type can be both. The compiler concludes that `Deserializer4` must be invariant in `T`.

```rs
    struct Deserializer4<T> {
        // some fields
        _t1: PhantomData<fn(T)>,
        _t2: PhantomData<fn() -> T>,
    }

    // alternative, but now need to have a lifetime on your type, which you probably do not want, hence the previous approach.
    // you can also do `PhantomData<*mut T>`
    struct Deserializer5<'a, T> {
        // some fields
        _t1: PhantomData<&'a mut T>,
    }
```

You can also do this:

```rs
    struct Deserializer2<T> {
        // some fields
        // this is covariant, which will support shortening lifetimes.
        _t1: PhantomData<fn() -> T>,
        // this is also covariant in T. However, Send and Sync are not auto-implemented for raw pointers.
        _t2: PhantomData<*const T>,
    }
```

If you used `PhantomData<*const T>` and if you have a type that contains a phantom data that contains a `*const T`, your outer type would not be `Send` and `Sync`. Normally auto-implemented for your types assuming all the members are. `PhantomData<fn() -> T>` is preferable to get those auto implements.

## Solving a real example: `strtok` (in Cpp)

PENDING

### TBD

```rs
// never do this, it's always UB
// &T -> *const T -> *mut T -> &mut T

// technically not UB
// &mut T -> *mut T -> *const T (covariance) -> *mut T -> &mut T (but need invariance)
```

## Further reading & Viewing

- [Crust of Rust: Subtyping and Variance][1] -- by [Jon Gjengset](https://www.youtube.com/@jonhoo)
- [Lifetime variance in Rust][3] -- with code examples

[1]: https://doc.rust-lang.org/nomicon/subtyping.html "Rustonomicon: Subtyping and Variance"
[2]: https://www.youtube.com/watch?v=iVYWDIW71jk "Crust of Rust: Subtyping and Variance by Jon Gjengset"
[3]: https://github.com/sunshowers-code/lifetime-variance "Lifetime variance in Rust"

## Acknowledgements

- [Jon Gjengset](https://www.youtube.com/@jonhoo) -- kudos as always to his amazing teaching style.
