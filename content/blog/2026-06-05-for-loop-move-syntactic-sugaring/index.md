+++
title = "TIP: Avoiding a Subtle Trap in Rust's For-loop Move Syntactic Sugaring"
# description = ""

[taxonomies]
tags = ["rust"] 
+++

When processing data as a read operation from any persistence layer of choice, bulk of the time is spent as:

- performing a query / search operation, pulling the data into a `HashMap<(_ /* id */, _ /* vector payload */)>`.
- transformations & compute
- formatting & transform to match output type
- return owned output value

It is always tempting to start like this

```rs
let collection1 = vec![1, 2, 3];
let collection2 = vec![collection1];

for inner in collection2 {
    // contents are _moved_ into the iterator here.
    for items in inner {}
}
```

For those unfamiliar, for-loops in Rust are sugar for [IntoIterator](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html), which means the contents are consumed by the block.

Curious readers will find [the rust docs covers its de-sugaring](https://doc.rust-lang.org/std/iter/index.html#for-loops-and-intoiterator) and we can see `fn into_iter(self)` within the trait takes `self` and not `&self`. It also makes sense that `Item` in `type IntoIter: Iterator<Item = Self::Item>` is also an owned value considering this is process inside a `loop`.

> _MUSING_: I do not want to think about `Item` being a shared reference, as this would not only complicate matters with life-times and/or alternatively involving `Pin<Box>` into this mix. Performance would be terrible, but it might be a fun brain teaser; thoughts?

Being consumed means that we cannot access them at a later point. This does not work:

```rs
// this is the naive way of accessing the inner iterator, which also causes a move due to
// Rust's for-loop being syntactic sugar of the IntoIterator trait.
for inner in collection2 {
    // contents are _moved_ into the iterator here.
    for items in inner {}
}
// collection2.iter();
// ^
// |__ we are now unable to iterate over it, due to the previous move.
```

Consider this approach, which I have extended to a `HashMap`.

```rs
for r in collection2.iter().flat_map(|r| r.iter()) {
    println!("value: {}", r);
}
// notice that we did not move the contents of the collection above.
let _ = collection2.iter().collect::<Vec<&Vec<i32>>>();

// the same applies for any other iterator; here's the same example with a HashMap:
let inner = HashMap::from([("".to_owned(), 8)]);
let collection = HashMap::<String, HashMap<String, i32>>::from([("a".to_string(), inner)]);

for (k, v) in collection.iter().flat_map(|(_, r)| r.iter()) {
    println!("value: {}", v);
}
```

I have applied this to my latest project which processes accounts within a ledger system. These are queried from the persistence layer into a `Vec<AccountQueryRecord>` but we have double nesting to group them by

- currency
- type of account such as `checking` or `savings` etc.

```rs
let mut accounts_hashmap: HashMap<
        Currency,
        HashMap<AccountType, (Vec<AccountQueryRecord>, AccountsTotalAsDecimal)>,
    > = HashMap::new();
```

The same approach above lets us partition the processed results and iterate over the original records fetched from the DB without needlessly re-allocating them on the heap.

Did you find this useful? Feel free to email me and share your thoughts!

If you made it this far, have a great day!
