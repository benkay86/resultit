//! Rust iterators return `Option<Item>`, but what happens if `Item` is a `Result` that could possibly be `Err`? This crate supplies iterator adapters to simplify working with so-called "fallible" iterators.  The supplied adapters are independent of each other; you can use the whole crate with `use resultit::*;` or just the iterator adapter traits you want with (for instance) `use resultit::FlattenResults`.  You are also free to take individual files (e.g. flatten_results.rs) and use them in your own source tree without depending on this crate.
//! 
//! Example:
//! 
//! ```
//! # struct Error1;
//! # impl std::fmt::Display for Error1 {
//! # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//! # 		write!(f, "Error1")
//! # 	}
//! # }
//! # impl std::fmt::Debug for Error1 {
//! # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//! # 		<Error1 as std::fmt::Display>::fmt(self, f)
//! # 	}
//! # }
//! # impl std::error::Error for Error1 { }
//! #
//! # struct Error2;
//! # impl std::fmt::Display for Error2 {
//! # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//! # 		write!(f, "Error2")
//! # 	}
//! # }
//! # impl std::fmt::Debug for Error2 {
//! # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//! # 		<Error2 as std::fmt::Display>::fmt(self, f)
//! # 	}
//! # }
//! # impl std::error::Error for Error2 { }
//! #
//! // Use the flatten_results() and stop_after_error() iterator adapters.
//! use resultit::FlattenResults;
//! use resultit::StopAfterError;
//!
//! // Use the TryError convenience/shorthand type.
//! use resultit::TryResult;
//! 
//! // Nested vector of results with different error types.
//! let v: Vec<Result<Vec<Result<i32, Error2>>, Error1>> = vec![
//! 	Ok(vec![Ok(1), Ok(2)]),
//! 	Ok(vec![Ok(3), Ok(4)]),
//! 	Ok(vec![Err(Error2{}), Ok(5)]),
//! 	Ok(vec![Ok(6), Ok(7)])
//! ];
//! 
//! // Flatten v, stopping after the first error.
//! let v: Vec<TryResult<i32>> = v.into_iter()
//! 	// Flatten the inner vectors.
//! 	.flatten_results()
//! 	// Flatten/erase error types by converting
//! 	// Result<Result<i32, Error2>, Error1> to Result<i32, E>
//! 	// where E is a boxed error trait object.
//! 	.map(|res| -> TryResult<_> { Ok(res??) } )
//! 	// Stop iterating after the first error is encountered.
//! 	.stop_after_error()
//! 	// Collect into vector v for this example.
//! 	// Could just as easily have done try_for_each() or any other adapter.
//! 	.collect();
//!
//! println!("{:?}", v);
//! // [Ok(1), Ok(2), Ok(3), Ok(4), Err(Error2)]
//! # assert_eq!(v.len(), 5);
//! # assert_eq!(*v[0].as_ref().unwrap(), 1);
//! # assert_eq!(*v[1].as_ref().unwrap(), 2);
//! # assert_eq!(*v[2].as_ref().unwrap(), 3);
//! # assert_eq!(*v[3].as_ref().unwrap(), 4);
//! # assert_eq!(v[4].is_err(), true);
//! #
//! # // Additional testing embedded in rustdoc.
//! # let v: Vec<Result<Vec<Result<i32, Error2>>, Error1>> = vec![
//! # 	Ok(vec![Ok(1), Ok(2)]),
//! # 	Ok(vec![Ok(3), Ok(4)]),
//! # 	Err(Error1{}),
//! # 	Ok(vec![Ok(5), Ok(6)])
//! # ];
//! # let v: Vec<TryResult<i32>> = v.into_iter()
//! # 	.flatten_results()
//! # 	.map(|res| -> TryResult<_> { Ok(res??) } )
//! # 	.stop_after_error()
//! # 	.collect();
//! # println!("{:?}", v);
//! # // [Ok(1), Ok(2), Ok(3), Ok(4), Err(Error1)]
//! # assert_eq!(v.len(), 5);
//! # assert_eq!(*v[0].as_ref().unwrap(), 1);
//! # assert_eq!(*v[1].as_ref().unwrap(), 2);
//! # assert_eq!(*v[2].as_ref().unwrap(), 3);
//! # assert_eq!(*v[3].as_ref().unwrap(), 4);
//! # assert_eq!(v[4].is_err(), true);
//! #
//! # let v: Vec<Result<Vec<Result<i32, Error2>>, Error1>> = vec![
//! # 	Ok(vec![Ok(1), Ok(2)]),
//! # 	Ok(vec![Ok(3), Ok(4)]),
//! # 	Ok(vec![Ok(5), Err(Error2{})]),
//! # 	Ok(vec![Ok(6), Ok(7)])
//! # ];
//! # let v: Vec<TryResult<i32>> = v.into_iter()
//! # 	.flatten_results()
//! # 	.map(|res| -> TryResult<_> { Ok(res??) } )
//! # 	.stop_after_error()
//! # 	.collect();
//! # println!("{:?}", v);
//! # // [Ok(1), Ok(2), Ok(3), Ok(4), Ok(5), Err(Error2)]
//! # assert_eq!(v.len(), 6);
//! # assert_eq!(*v[0].as_ref().unwrap(), 1);
//! # assert_eq!(*v[1].as_ref().unwrap(), 2);
//! # assert_eq!(*v[2].as_ref().unwrap(), 3);
//! # assert_eq!(*v[3].as_ref().unwrap(), 4);
//! # assert_eq!(*v[4].as_ref().unwrap(), 5);
//! # assert_eq!(v[5].is_err(), true);
//! ```

// Re-export iterator adapter traits from submodules.
pub mod flatten_results;
pub use flatten_results::FlattenResults;
pub mod stop_after_error;
pub use stop_after_error::StopAfterError;

/// Shorthand for a Result with a boxed error trait.
/// Provided for convenience, not a dependency of any submodule.
///
/// Particularly useful for flattening iterators of nested results by
/// flattening/combining/erasing the error types.  See the the
/// [crate level documentation](crate) for an example of how to do this.  A
/// simpler example of what this type does follows below:
/// 
/// ```
/// // Trivial case in which we return just one error type.
/// fn parse1(num: &str) -> Result<i32, <i32 as std::str::FromStr>::Err> {
/// 	return num.parse();
/// }
///
/// // What if our function can return more than one error type?
/// // We can return a boxed error trait to erase the type of the error.
/// // We can use then use the ? (aka try) operator to propagate errors.
/// // For maximum compatibility with threaded programs,
/// // the error should also implement send and sync.
/// fn parse2(num: &str) -> Result<i32, std::boxed::Box<
/// 	dyn std::error::Error
/// 	+ std::marker::Send
/// 	+ std::marker::Sync
/// >> {
/// 	// do_something_fallible()?;
/// 	let parsed_num = (num.parse())?;
///		return Ok(parsed_num);
/// }
/// 
/// // Same as parse2() but using TryResult<i32> as shorthand.
/// fn parse3(num: &str) -> resultit::TryResult<i32> {
/// 	// do_something_fallible()?;
/// 	let parsed_num = (num.parse())?;
///		return Ok(parsed_num);
/// }
/// # assert_eq!(parse1("42").unwrap(), 42);
/// # assert_eq!(parse2("42").unwrap(), 42);
/// # assert_eq!(parse3("42").unwrap(), 42);
/// ```
pub type TryResult<T> = std::result::Result<
	T,
	std::boxed::Box< dyn
		std::error::Error   // must implement Error to satisfy Try
		+ std::marker::Send // needed to move errors between threads
		+ std::marker::Sync // needed to move errors between threads
	>
>;
