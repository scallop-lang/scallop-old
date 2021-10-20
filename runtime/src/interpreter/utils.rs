use std::sync::Arc;

pub type CompString = Arc<String>;

pub(crate) fn gallop<T>(mut slice: &[T], mut cmp: impl FnMut(&T) -> bool) -> &[T] {
  // if empty slice, or already >= element, return
  if !slice.is_empty() && cmp(&slice[0]) {
    let mut step = 1;
    while step < slice.len() && cmp(&slice[step]) {
      slice = &slice[step..];
      step <<= 1;
    }

    step >>= 1;
    while step > 0 {
      if step < slice.len() && cmp(&slice[step]) {
        slice = &slice[step..];
      }
      step >>= 1;
    }

    slice = &slice[1..]; // advance one, as we always stayed < value
  }

  slice
}
