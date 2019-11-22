# ResultIt

Rust iterators return `Option<Item>`, but what happens if `Item` is a `Result` that could possibly be `Err`? This crate supplies iterator adapters to simplify working with so-called "fallible" iterators.  The supplied adapters are independent of each other; you can use the whole crate with `use resultit::*;` or just the iterator adapter traits you want with (for instance) `use resultit::FlattenResults`.  You are also free to take individual files (e.g. flatten_results.rs) and use them in your own source tree without depending on this crate.  See the [documentation](https://docs.rs/resultit/) for more information and examples.

## Background

Rust iterators implement a function [next()](https://doc.rust-lang.org/std/iter/trait.Iterator.html#tymethod.next) that returns `Option<Item>` where `Item` is the type over which the iterator is iterating.  For example:

```
let v: Vec<i32> = vec![1, 2, 3];
let iter = v.into_iter(); // satisfies trait bound Iterator<Item=i32>
while let Some(i) = iter.next() {
	println!("{}", i); // i has type i32
}
// Output:
// 1
// 2
// 3
```

In the somewhat contrived example above, `iter` is an iterator over the vector `v` and satisfies the trait bound `Iterator<Item=i32>`.  The `while let` statement executes as long as `iter.next()` returns `Some(i)` where `i` is the next integer in the vector.  After `iter.next()` returns `Some(3)`, the next call to `iter.next()` returns `None` and the `while let` loop ends.  Usually you would use the more idiomatic constructs `for i in vec` or `for i in iter` or `iter.for_each(|i| println!("{}", i)` and the `Option<Item>` would be unwrapped for you automatically.

As we all know, not everything in programming always goes as planned.  This crate exists to help with cases where `Item` is a `Result` that could possibly contain an `Err`.  Using such an iterator is challenging in particular when:

* Flattening an iterator of results, see `FlattenResults` and `flatten_results()`
* Flattening or erasing error types in an iterator of nested results, see `TryError`
* Stopping iteration after the first error, see `StopAfterError` and `stop_after_error()`

Please see the [rustdoc documentation](https://docs.rs/resultit/) for details of how to use each of the above tools.

## Example

There are multiple didactic examples in the [documentation](https://docs.rs/resultit/).  A real-world example using the [glob](https://crates.io/crates/glob) crate follows:

```
// Use the resultit crate.
use resultit::*;

// Look for image files with different extensions.
let glob_patterns = vec!["*.png", "*.jpg"];

// Print out a list of matching image files.
glob_patterns.into_iter()
	// Attempt to convert each glob pattern into an iterator over matching paths.
	.map(|pattern| -> glob::glob(&pattern))
	
	// Flatten over each inner iterator over matching paths.
	.flatten_results()
	
	// Flatten/erase the nested error types PatternError and GlobError.
	.map(|path| -> lib::TryResult<_> { Ok(path??) })
	
	// Stop iteration after the first error is encountered.
	.stop_after_error()
	
	// Generate some output, propagating errors up the stack.
	.try_for_each(|path| -> TryResult<()> {
			Ok(println!("Found image: {:?}", path?))
	});
```
