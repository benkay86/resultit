//! Iterator adapter to stop iteration after the first error is encountered.
//! See documentation for [StopAfterError] for details.

/// Iterator adapter to stop iteration after the first error is encountered.  Lazily yields each item in the iterator up to and including the first error.  Subsequent calls to [Iterator::next()](std::iter::Iterator::next()) return [None](std::option::Option::None).  Use this trait to enable the [stop_after_error()](StopAfterError::stop_after_error()) function for iterators.
///
/// Example:
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
/// // Use the StopAfterError trait to enable stop_after_error() on iterators.
/// use resultit::StopAfterError;
/// 
/// // Vector of results in which one result is an error.
/// let mut v: Vec<Result<i32, MyError>> = vec![
/// 	Ok(1),
/// 	Ok(2),
/// 	Err(MyError{}),
/// 	Ok(3)
/// ];
/// 
/// // Collection continues even after encountering the error.
/// v = v.into_iter().collect();
/// # assert_eq!(v.len(), 4);
/// println!("{:?}", v);
/// // [Ok(1), Ok(2), Err(MyError), Ok(3)]
/// 
/// // Use stop_after_error() to stop iteration after the first error.
/// v = v.into_iter().stop_after_error().collect();
/// println!("{:?}", v);
/// // [Ok(1), Ok(2), Err(MyError)]
/// # assert_eq!(v.len(), 3);
/// # assert_eq!(*v[0].as_ref().unwrap(), 1);
/// # assert_eq!(*v[1].as_ref().unwrap(), 2);
/// # assert_eq!(v[2].is_err(), true);
/// ```
pub trait StopAfterError {
	/// Iterator adapter to stop iteration after the first error is encountered.
	/// See documentation for [StopAfterError] for details.
	fn stop_after_error<O, E>(self) -> StopAfterErrorIter<Self>
	where
		Self: Iterator<Item=Result<O, E>> + Sized;
}

// Blanket implementation of the StopAfterError trait for all iterators.
// This is what enables us to call stop_after_error() on any iterator.
impl<It> StopAfterError for It
where
	It: Iterator + Sized
{
	fn stop_after_error<O, E>(self) -> StopAfterErrorIter<Self>
	where
		Self: Iterator<Item=Result<O, E>> + Sized
	{
		// Initialize StopAfterErrorIter with the iterator we were called on and the error flag initially set to false.
		StopAfterErrorIter{iter: self, error: false}
	}
}

/// Iterator returned by [stop_after_error()](StopAfterError::stop_after_error()).  You should not need to use this directly.  See the documentation for [StopAfterError] for intended use.
pub struct StopAfterErrorIter<It> {
	// Iterator we are wrapping.
	iter: It,
	
	// Eror flag, set to true upon the first error.
	error: bool
}

impl<It, O, E> Iterator for StopAfterErrorIter<It>
where
	It: Iterator<Item=Result<O, E>>
{
	type Item = Result<O, E>;
	
	// Return the next item in iter until after the first error.
	// Then return None.
	fn next(&mut self) -> Option<Self::Item> {
		match self.error {
			true => None,
			false => {
				match self.iter.next() {
					None => None,
					Some(result) => {
						match result {
							Ok(o) => Some(Ok(o)),
							Err(e) => {
								// Error detected, set the error flag to true.
								self.error = true;
								
								// Return the error.  Now that the error flag is true, the next call to this function will return None.
								Some(Err(e))
							}
						}
					}
				}
			}
		}
	}
}
