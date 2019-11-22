//! Iterator adapter to flatten an `Iterator<Item> where: Item = Result<IntoIterator,_>`.
//! See documentation for [FlattenResults] for details.
//! 
//! Based on solution proposed by redditor [earthengine](https://www.reddit.com/user/earthengine/) in [this post](https://www.reddit.com/r/rust/comments/9u6846/rust_puzzle_flatten_a_nested_iterator_of_results/) and on the [rust playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2015&gist=30e0fb57a8ba63777052a344054b22c0).
// 
// Implementation is non-trivial because trait methods cannot return impl trait, see the relevant RFC:
// https://github.com/rust-lang/rfcs/blob/master/text/1522-conservative-impl-trait.md
// This necessitates the rather complex and manually-defined return type of FlattenResults::flatten_result().
// To further complicated matters, rust rlosures are anonymous types and therefore cannot be expressed in a manually-defined return type.  Instead we declare module-private functions flatten() and wrap_result() below, and we use pointers to these functions (which do have a type) instead of closures.

// Module-private function to flatten an iterator of results, see earthengine's solution for an explanation.
fn flatten<InnerIter, T, Error>(res: Result<InnerIter, Error>) -> std::iter::Chain<std::iter::Map<std::iter::Flatten<std::option::IntoIter<InnerIter>>,fn(T)->Result<T, Error>>, std::option::IntoIter<std::result::Result<T, Error>>>
where
	InnerIter: IntoIterator<Item = T>
{
	let (v,r) = match res {
		Ok(v) => (Some(v), None),
		Err(e) => (None, Some(Err(e)))
	};
	v.into_iter().flatten().map(wrap_result as fn(T)->Result<T, Error>).chain(r)
}

// Function pointer to pass to map() adapter in flatten() above.
// Wraps a type T in a Result<T,_>.
fn wrap_result<T,E>(t: T) -> Result<T, E> {
	Ok(t)
}

/// Module-level free-standing function analog to [flatten_results()](FlattenResults::flatten_results()) that can be called without using the [FlattenResults] trait.  It is more idiomatic to use the iterator adapter and call [flatten_results()](FlattenResults::flatten_results()) instead.
///
/// For example:
///
/// ```
/// # struct MyError;
/// # impl std::fmt::Display for MyError {
/// # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// # 		write!(f, "MyError")
/// # 	}
/// # }
/// # impl std::fmt::Debug for MyError {
/// # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// # 		<MyError as std::fmt::Display>::fmt(self, f)
/// # 	}
/// # }
/// # impl std::error::Error for MyError { }
/// // Vector of results, each of which contains its own vector of numbers.
/// let v: Vec<Result<Vec<i32>, MyError>> = vec![
/// 	Ok(vec![1, 2]),
/// 	Ok(vec![3, 4]),
/// 	Err(MyError{}),
/// 	Ok(vec![5, 6])
/// ];
///
/// // Flatten the vector of results. 
/// // Propagates the error instead of panicking.
/// let v: Vec<Result<i32, MyError>> =
/// 	resultit::flatten_results::transform(v.into_iter())
/// 	.collect();
/// 
/// println!("{:?}", v);
/// // [Ok(1), Ok(2), Ok(3), Ok(4), Err(MyError), Ok(5), Ok(6)]
/// # assert_eq!(*v[0].as_ref().unwrap(), 1);
/// # assert_eq!(*v[1].as_ref().unwrap(), 2);
/// # assert_eq!(*v[2].as_ref().unwrap(), 3);
/// # assert_eq!(*v[3].as_ref().unwrap(), 4);
/// # assert_eq!(v[4].is_err(), true);
/// # assert_eq!(*v[5].as_ref().unwrap(), 5);
/// # assert_eq!(*v[6].as_ref().unwrap(), 6);
/// ```
pub fn transform<OuterIter, InnerIter, T, Error>(outer_iter: OuterIter) -> std::iter::FlatMap<OuterIter, std::iter::Chain<std::iter::Map<std::iter::Flatten<std::option::IntoIter<InnerIter>>,fn(T)->Result<T, Error>>, std::option::IntoIter<std::result::Result<T, Error>>>, /* type we are mapping to */ fn(Result<InnerIter, Error>)->std::iter::Chain<std::iter::Map<std::iter::Flatten<std::option::IntoIter<InnerIter>>,fn(T)->Result<T, Error>>, std::option::IntoIter<std::result::Result<T, Error>>>>
	where
		OuterIter: Iterator<Item = Result<InnerIter, Error>> + Sized,
		InnerIter: IntoIterator<Item = T>
	{
		outer_iter.flat_map(flatten)
	}

/// Iterator adapter to flatten an `Iterator<Item> where: Item = Result<IntoIterator,_>`.  This is needed because the standard [flatten()](std::iter::Iterator::flatten()) adapter only works on an iterator of iterators and does not work on an iterator of results.
/// 
/// Example of what not to do:
///
/// ```should_panic
/// # struct MyError;
/// # impl std::fmt::Display for MyError {
/// # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// # 		write!(f, "MyError")
/// # 	}
/// # }
/// # impl std::fmt::Debug for MyError {
/// # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// # 		<MyError as std::fmt::Display>::fmt(self, f)
/// # 	}
/// # }
/// # impl std::error::Error for MyError { }
/// // Vector of results, each of which contains its own vector of numbers.
/// let v: Vec<Result<Vec<i32>, MyError>> = vec![
/// 	Ok(vec![1, 2]),
/// 	Ok(vec![3, 4]),
/// 	Err(MyError{}),
/// 	Ok(vec![5, 6])
/// ];
/// 
/// // Panics after printing the number 4.  Probably not what you want!
/// v.into_iter()
/// 	.map(|res| res.unwrap())
/// 	.flatten()
/// 	.for_each(|i| println!("{}", i));
/// ```
/// 
/// Instead of [flatten()](std::iter::Iterator::flatten()), use [flatten_results()](FlattenResults::flatten_results()) to flatten each `Ok(IntoInterator)` and pass through the errors.
/// 
/// ```
/// # struct MyError;
/// # impl std::fmt::Display for MyError {
/// # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// # 		write!(f, "MyError")
/// # 	}
/// # }
/// # impl std::fmt::Debug for MyError {
/// # 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// # 		<MyError as std::fmt::Display>::fmt(self, f)
/// # 	}
/// # }
/// # impl std::error::Error for MyError { }
/// // Use the FlattenResults trait to enable flatten_results() on iterators.
/// use resultit::FlattenResults;
///
/// // Vector of results, each of which contains its own vector of numbers.
/// let v: Vec<Result<Vec<i32>, MyError>> = vec![
/// 	Ok(vec![1, 2]),
/// 	Ok(vec![3, 4]),
/// 	Err(MyError{}),
/// 	Ok(vec![5, 6])
/// ];
/// 
/// // Flatten the vector of results.
/// // Propagates the error instead of panicking.
/// let v: Vec<Result<i32, MyError>> = v.into_iter()
/// 	.flatten_results()
///		.collect();
/// 
/// println!("{:?}", v);
/// // [Ok(1), Ok(2), Ok(3), Ok(4), Err(MyError), Ok(5), Ok(6)]
/// # assert_eq!(*v[0].as_ref().unwrap(), 1);
/// # assert_eq!(*v[1].as_ref().unwrap(), 2);
/// # assert_eq!(*v[2].as_ref().unwrap(), 3);
/// # assert_eq!(*v[3].as_ref().unwrap(), 4);
/// # assert_eq!(v[4].is_err(), true);
/// # assert_eq!(*v[5].as_ref().unwrap(), 5);
/// # assert_eq!(*v[6].as_ref().unwrap(), 6);
/// ```
pub trait FlattenResults {
	/// Iterator adapter to flatten an `Iterator<Item> where: Item = Result<IntoIterator,_>`.  See the documentation of [FlattenResults].
	fn flatten_results<InnerIter, Error, T>(self) -> std::iter::FlatMap<Self, std::iter::Chain<std::iter::Map<std::iter::Flatten<std::option::IntoIter<InnerIter>>,fn(T)->Result<T, Error>>, std::option::IntoIter<std::result::Result<T, Error>>>, /* type we are mapping to */ fn(Result<InnerIter, Error>)->std::iter::Chain<std::iter::Map<std::iter::Flatten<std::option::IntoIter<InnerIter>>,fn(T)->Result<T, Error>>, std::option::IntoIter<std::result::Result<T, Error>>>>
	where
		Self: Iterator<Item = Result<InnerIter, Error>> + Sized,
		InnerIter: IntoIterator<Item = T>;
}

// Blanket implementation of the FlattenREsults trait for all iterators.
// This is what enables us to call flatten_results() on any iterator.
impl<It> FlattenResults for It
where
	It: Iterator + Sized
{
	fn flatten_results<InnerIter, Error, T>(self) -> std::iter::FlatMap<Self, std::iter::Chain<std::iter::Map<std::iter::Flatten<std::option::IntoIter<InnerIter>>,fn(T)->Result<T, Error>>, std::option::IntoIter<std::result::Result<T, Error>>>, /* type we are mapping to */ fn(Result<InnerIter, Error>)->std::iter::Chain<std::iter::Map<std::iter::Flatten<std::option::IntoIter<InnerIter>>,fn(T)->Result<T, Error>>, std::option::IntoIter<std::result::Result<T, Error>>>>
	where
		Self: Iterator<Item = Result<InnerIter, Error>> + Sized,
		InnerIter: IntoIterator<Item = T>
	{
		// Delegate to flatten_results::transform() to avoid code duplication.
		transform(self)
	}
}
