//! PackedStructContainer: A type-safe container for Pod structs.
//!
//! Provides a high-level interface over RawBytesContainer for working with
//! arrays of Pod types, supporting both in-memory and memory-mapped storage.

use bytemuck::Pod;
//use bytemuck_derive::Pod;
//use bytemuck_derive::Zeroable;
use raw_bytes_container::RawBytesContainer;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A container of packed Pod structs.
///
/// Can be backed by in-memory storage or memory-mapped files.
/// Provides zero-cost abstraction over byte arrays with type safety.
///
/// # Example
/// ```
/// use packed_struct_container::PackedStructContainer;
/// use bytemuck::{Pod, Zeroable};
/// use bytemuck_derive::Pod;
/// use bytemuck_derive::Zeroable;
///
/// #[repr(C)]
/// #[derive(Clone, Copy, Pod, Zeroable)]
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// let mut container = PackedStructContainer::new();
/// container.append(&[Point { x: 1.0, y: 2.0 }]).unwrap();
/// assert_eq!(container[0].x, 1.0);
/// ```
#[derive(Debug)]
pub struct PackedStructContainer<T: Pod + Copy> {
    storage: RawBytesContainer<u8>,
    _marker: PhantomData<T>,
}

impl<T: Pod + Copy> PackedStructContainer<T> {
    /// Create an empty in-memory container.
    pub fn new() -> Self {
        Self::from_slice(&[])
    }

    /// Create an in-memory container with pre-allocated capacity for `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        let byte_capacity = capacity * std::mem::size_of::<T>();
        Self {
            storage: RawBytesContainer::from_vec(Vec::with_capacity(byte_capacity)),
            _marker: PhantomData,
        }
    }

    /// Create from a slice (in-memory).
    pub fn from_slice(data: &[T]) -> Self {
        Self::validate_alignment();
        let bytes = bytemuck::cast_slice(data).to_vec();
        Self {
            storage: RawBytesContainer::from_vec(bytes),
            _marker: PhantomData,
        }
    }

    /// Create from values (convenience method).
    pub fn from_values(values: &[T]) -> Self {
        Self::from_slice(values)
    }

    /// Open a memory-mapped file read-only.
    pub fn open_mmap_read<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, raw_bytes_container::ContainerError> {
        Self::validate_alignment();
        Ok(Self {
            storage: RawBytesContainer::open_mmap_read(path)?,
            _marker: PhantomData,
        })
    }

    /// Open a memory-mapped file read-write.
    pub fn open_mmap_rw<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, raw_bytes_container::ContainerError> {
        Self::validate_alignment();
        Ok(Self {
            storage: RawBytesContainer::open_mmap_rw(path)?,
            _marker: PhantomData,
        })
    }

    /// Validate that T has proper alignment for byte-level casting.
    fn validate_alignment() {
        // bytemuck already validates this at compile time via Pod trait,
        // but we add a runtime check for extra safety
        assert!(
            std::mem::align_of::<T>() <= 8,
            "Type alignment too strict for safe casting"
        );
    }

    /// Returns the number of elements in the container.
    pub fn len(&self) -> usize {
        self.storage.as_slice().len() / std::mem::size_of::<T>()
    }

    /// Returns true if the container is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the capacity in elements before reallocation. (NOT CRRECT)
    /// currently returns len() in bytes / size_of::<T>()
    /// it can't be implemented accurately without access to the underlying Vec's capacity.
    // pub fn capacity(&self) -> usize {
    //     self.storage.as_slice().len() / std::mem::size_of::<T>()
    // }

    /// Access as slice of T.
    pub fn as_slice(&self) -> &[T] {
        bytemuck::cast_slice(self.storage.as_slice())
    }

    /// Access as mutable slice if storage is writable.
    ///
    /// Returns `None` if the storage is read-only (e.g., read-only mmap).
    pub fn as_slice_mut(&mut self) -> Option<&mut [T]> {
        Some(bytemuck::cast_slice_mut(self.storage.as_slice_mut()?))
    }

    /// Get element by index.
    pub fn get(&self, index: usize) -> Option<T> {
        self.as_slice().get(index).copied()
    }

    /// Get mutable reference to element by index.
    ///
    /// Returns `None` if index is out of bounds or storage is read-only.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.as_slice_mut()?.get_mut(index)
    }

    /// Append new elements (in-memory only).
    ///
    /// # Errors
    /// Returns an error if the storage is read-only or cannot be resized.
    pub fn append(&mut self, new: &[T]) -> Result<(), raw_bytes_container::ContainerError> {
        let new_bytes = bytemuck::cast_slice(new);
        self.storage.append(new_bytes)
    }

    /// Append a single element.
    ///
    /// # Errors
    /// Returns an error if the storage is read-only or cannot be resized.
    pub fn push(&mut self, value: T) -> Result<(), raw_bytes_container::ContainerError> {
        self.append(&[value])
    }

    /// Extend from an iterator.
    ///
    /// # Errors
    /// Returns an error if the storage is read-only or cannot be resized.
    pub fn extend<I>(&mut self, iter: I) -> Result<(), raw_bytes_container::ContainerError>
    where
        I: IntoIterator<Item = T>,
    {
        let values: Vec<T> = iter.into_iter().collect();
        self.append(&values)
    }

    /// Clear all elements (in-memory only).
    ///
    /// # Errors
    /// Returns an error if the storage is read-only.
    pub fn clear(&mut self) -> Result<(), raw_bytes_container::ContainerError> {
        self.storage.resize(0, 0)
    }

    /// Flush changes to disk (for memory-mapped files).
    pub fn flush(&self) -> Result<(), raw_bytes_container::ContainerError> {
        self.storage.flush()
    }

    /// Expose underlying storage for advanced use.
    pub fn storage(&self) -> &RawBytesContainer<u8> {
        &self.storage
    }

    /// Mutable access to underlying storage for advanced use.
    pub fn storage_mut(&mut self) -> &mut RawBytesContainer<u8> {
        &mut self.storage
    }

    /// Returns an iterator over the elements.
    pub fn iter(&self) -> std::iter::Copied<std::slice::Iter<'_, T>> {
        self.as_slice().iter().copied()
    }
}

impl<T: Pod + Copy> Default for PackedStructContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Deref to slice for ergonomic access.
///
/// Allows using the container like a slice: `container[i]`, `container.len()`, etc.
impl<T: Pod + Copy> Deref for PackedStructContainer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// DerefMut for mutable slice access.
///
/// # Panics
/// Panics if the storage is read-only (e.g., read-only memory-mapped file).
/// Use `as_slice_mut()` for non-panicking access.
impl<T: Pod + Copy> DerefMut for PackedStructContainer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
            .expect("Cannot mutably dereference read-only storage")
    }
}

/// Iterator support - iterate over copies of elements.
impl<'a, T: Pod + Copy> IntoIterator for &'a PackedStructContainer<T> {
    type Item = T;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck_derive::Pod;
    use bytemuck_derive::Zeroable;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
    struct Point {
        x: f32,
        y: f32,
    }

    #[test]
    fn test_new_and_push() {
        let mut container = PackedStructContainer::new();
        assert_eq!(container.len(), 0);
        assert!(container.is_empty());

        container.push(Point { x: 1.0, y: 2.0 }).unwrap();
        container.push(Point { x: 3.0, y: 4.0 }).unwrap();

        assert_eq!(container.len(), 2);
        assert_eq!(container[0].x, 1.0);
        assert_eq!(container[1].y, 4.0);
    }

    #[test]
    fn test_from_slice() {
        let points = [Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];
        let container = PackedStructContainer::from_slice(&points);

        assert_eq!(container.len(), 2);
        assert_eq!(container[0], points[0]);
    }

    #[test]
    fn test_append() {
        let mut container = PackedStructContainer::new();
        let points = [Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];

        container.append(&points).unwrap();
        assert_eq!(container.len(), 2);
    }

    #[test]
    fn test_extend() {
        let mut container = PackedStructContainer::new();
        let points = vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];

        container.extend(points).unwrap();
        assert_eq!(container.len(), 2);
    }

    #[test]
    fn test_deref() {
        let mut container = PackedStructContainer::from_slice(&[
            Point { x: 1.0, y: 2.0 },
            Point { x: 3.0, y: 4.0 },
        ]);

        // Use as slice
        assert_eq!(container.len(), 2);

        // Mutable access via DerefMut
        container[0].x = 10.0;
        assert_eq!(container[0].x, 10.0);
    }

    #[test]
    fn test_iterator() {
        let points = [Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];
        let container = PackedStructContainer::from_slice(&points);

        let collected: Vec<_> = container.iter().collect();
        assert_eq!(collected, points);

        // Also test IntoIterator
        let collected2: Vec<_> = (&container).into_iter().collect();
        assert_eq!(collected2, points);
    }

    #[test]
    fn test_get_mut() {
        let mut container = PackedStructContainer::from_slice(&[Point { x: 1.0, y: 2.0 }]);

        if let Some(point) = container.get_mut(0) {
            point.x = 100.0;
        }

        assert_eq!(container[0].x, 100.0);
    }

    #[test]
    fn test_with_capacity() {
        let mut container = PackedStructContainer::<Point>::with_capacity(100);
        assert_eq!(container.len(), 0);

        // Add some elements to verify it works
        for i in 0..10 {
            container
                .push(Point {
                    x: i as f32,
                    y: i as f32 * 2.0,
                })
                .unwrap();
        }
        assert_eq!(container.len(), 10);
    }

    #[test]
    fn test_clear() {
        let mut container = PackedStructContainer::from_slice(&[
            Point { x: 1.0, y: 2.0 },
            Point { x: 3.0, y: 4.0 },
        ]);

        assert_eq!(container.len(), 2);
        container.clear().unwrap();
        assert_eq!(container.len(), 0);
        assert!(container.is_empty());
    }
}
