// mtf_api/src/dynamic.rs

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::NonNull;

use bytemuck::{Pod, from_bytes};
use mtf::{FieldDef, MTFError, Result, TypeDef, read_mtf, read_string};

/// A handle to a single field in a struct.
///
/// Provides a builder-style API for modifying field values.
pub struct FieldHandle<'a, T> {
    ptr: Option<NonNull<T>>,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> FieldHandle<'a, T> {
    /// Create an empty handle (no field found).
    pub fn none() -> Self {
        Self {
            ptr: None,
            _phantom: PhantomData,
        }
    }

    /// Create a handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid, properly aligned, and point to initialized data.
    unsafe fn from_ptr(p: *mut T) -> Self {
        Self {
            ptr: NonNull::new(p),
            _phantom: PhantomData,
        }
    }

    /// Returns true if the handle points to a valid field.
    pub fn is_some(&self) -> bool {
        self.ptr.is_some()
    }

    /// Get an immutable reference to the field value.
    pub fn get(&self) -> Option<&T> {
        self.ptr.map(|p| unsafe { p.as_ref() })
    }

    /// Get a mutable reference to the field value.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.ptr.map(|mut p| unsafe { p.as_mut() })
    }

    /// Set the field value.
    pub fn set(&mut self, v: T) -> &mut Self {
        if let Some(p) = self.ptr {
            unsafe { *p.as_ptr() = v }
        }
        self
    }

    /// Add to the field value (requires AddAssign).
    pub fn add(&mut self, v: T) -> &mut Self
    where
        T: std::ops::AddAssign + Copy,
    {
        if let Some(mut p) = self.ptr {
            unsafe { *p.as_mut() += v }
        }
        self
    }

    /// Subtract from the field value (requires SubAssign).
    pub fn sub(&mut self, v: T) -> &mut Self
    where
        T: std::ops::SubAssign + Copy,
    {
        if let Some(mut p) = self.ptr {
            unsafe { *p.as_mut() -= v }
        }
        self
    }

    /// Apply a closure to modify the field value.
    pub fn apply<F: FnOnce(&mut T)>(&mut self, f: F) -> &mut Self {
        if let Some(mut p) = self.ptr {
            unsafe { f(p.as_mut()) }
        }
        self
    }
}

/// Dynamic access to a slice of structs with MTF metadata.
///
/// Allows field access by name at runtime, useful for:
/// - Generic tooling and editors
/// - Serialization/deserialization
/// - Dynamic queries
pub struct DynamicContainer {
    data: Vec<u8>,
    type_def: TypeDef,
    strings: Vec<u8>,
    struct_size: usize,
    field_map: HashMap<String, FieldDef>,
}

impl DynamicContainer {
    /// Construct from raw data and a complete MTF blob.
    pub fn from_raw(data: Vec<u8>, blob: &[u8]) -> Result<Self> {
        let (types, strings) = read_mtf(blob)?;

        let type_def = types.into_iter().next().ok_or(MTFError::UnexpectedEof)?;

        let struct_size = (type_def.size_bits as usize + 7) / 8; // Round up to bytes

        // Precompute field name -> FieldDef map for fast lookups
        let mut field_map = HashMap::new();
        for f in &type_def.fields {
            let name = read_string(strings, f.name_offset)?;
            field_map.insert(name.to_string(), f.clone());
        }

        Ok(Self {
            data,
            type_def,
            strings: strings.to_vec(),
            struct_size,
            field_map,
        })
    }

    /// Construct directly from a file containing MTF-embedded data.
    ///
    /// Expects format: [DATA][METADATA_SIZE: u32][METADATA]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let len = file.metadata()?.len();

        if len < 4 {
            return Err(MTFError::UnexpectedEof);
        }

        // Read metadata size from the end
        file.seek(SeekFrom::End(-4))?;
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf)?;
        let metadata_size = u32::from_le_bytes(buf) as u64;

        if metadata_size + 4 > len {
            return Err(MTFError::UnexpectedEof);
        }

        // Calculate where data ends and metadata begins
        let data_len = len - metadata_size - 4;

        // Read data
        file.seek(SeekFrom::Start(0))?;
        let mut data = vec![0u8; data_len as usize];
        file.read_exact(&mut data)?;

        // Read metadata blob
        let mut blob = vec![0u8; metadata_size as usize];
        file.read_exact(&mut blob)?;

        Self::from_raw(data, &blob)
    }

    /// Returns the number of structs in the container.
    pub fn len(&self) -> usize {
        if self.struct_size == 0 {
            0
        } else {
            self.data.len() / self.struct_size
        }
    }

    /// Returns true if the container is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the type name.
    pub fn type_name(&self) -> Result<&str> {
        read_string(&self.strings, self.type_def.name_offset)
    }

    /// List all field names.
    pub fn field_names(&self) -> Vec<String> {
        self.field_map.keys().cloned().collect()
    }

    /// Immutable access to a field of a struct at index.
    pub fn field<T: Pod>(&self, index: usize, field_name: &str) -> Option<&T> {
        // Bounds check
        if index >= self.len() {
            return None;
        }

        // Get field definition
        let field = self.field_map.get(field_name)?;

        // Check size matches
        let field_size = (field.size_bits as usize + 7) / 8;
        if field_size != std::mem::size_of::<T>() {
            return None;
        }

        // Check alignment
        let field_offset = (field.offset_bits / 8) as usize;
        if field_offset % std::mem::align_of::<T>() != 0 {
            return None; // Misaligned
        }

        // Calculate struct position
        let struct_start = index * self.struct_size;
        let field_start = struct_start + field_offset;
        let field_end = field_start + field_size;

        // Get field slice
        let field_slice = self.data.get(field_start..field_end)?;

        Some(from_bytes(field_slice))
    }

    /// Mutable access to a field of a struct at index.
    pub fn field_mut<T: Pod>(&mut self, index: usize, field_name: &str) -> FieldHandle<'_, T> {
        // Bounds check
        if index >= self.len() {
            return FieldHandle::none();
        }

        // Get field definition
        let field = match self.field_map.get(field_name) {
            Some(f) => f,
            None => return FieldHandle::none(),
        };

        // Check size matches
        let field_size = (field.size_bits as usize + 7) / 8;
        if field_size != std::mem::size_of::<T>() {
            return FieldHandle::none();
        }

        // Check alignment
        let field_offset = (field.offset_bits / 8) as usize;
        if field_offset % std::mem::align_of::<T>() != 0 {
            return FieldHandle::none(); // Misaligned
        }

        // Calculate struct position
        let struct_start = index * self.struct_size;
        let field_start = struct_start + field_offset;
        let field_end = field_start + field_size;

        // Get mutable field slice
        let field_slice = match self.data.get_mut(field_start..field_end) {
            Some(s) => s,
            None => return FieldHandle::none(),
        };

        let ptr = field_slice.as_mut_ptr() as *mut T;
        unsafe { FieldHandle::from_ptr(ptr) }
    }

    /// Get raw byte data.
    pub fn raw(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable raw byte data.
    pub fn raw_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Iterator over struct indices.
    pub fn iter(&self) -> DynamicContainerIter<'_> {
        DynamicContainerIter {
            container: self,
            index: 0,
        }
    }
}

/// Iterator over the container structs (yields indices).
pub struct DynamicContainerIter<'a> {
    container: &'a DynamicContainer,
    index: usize,
}

impl<'a> Iterator for DynamicContainerIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.container.len() {
            let idx = self.index;
            self.index += 1;
            Some(idx)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.container.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for DynamicContainerIter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock data for testing
    fn create_test_blob() -> Vec<u8> {
        // A minimal MTF blob for testing
        let mut blob = Vec::new();
        blob.extend_from_slice(b"MTF\0"); // Magic
        blob.extend_from_slice(&1u32.to_le_bytes()); // Version
        blob.extend_from_slice(&1u32.to_le_bytes()); // Type count
        blob.extend_from_slice(&0u32.to_le_bytes()); // Type name offset
        blob.extend_from_slice(&64u32.to_le_bytes()); // Size bits (8 bytes)
        blob.extend_from_slice(&2u32.to_le_bytes()); // Field count

        // Field 1: "x" at offset 0, 32 bits
        blob.extend_from_slice(&5u32.to_le_bytes()); // Name offset
        blob.extend_from_slice(&0u32.to_le_bytes()); // Offset bits
        blob.extend_from_slice(&32u32.to_le_bytes()); // Size bits

        // Field 2: "y" at offset 32, 32 bits
        blob.extend_from_slice(&7u32.to_le_bytes()); // Name offset
        blob.extend_from_slice(&32u32.to_le_bytes()); // Offset bits
        blob.extend_from_slice(&32u32.to_le_bytes()); // Size bits

        // String table size
        blob.extend_from_slice(&9u32.to_le_bytes());

        // String table: "Test\0x\0y\0"
        blob.extend_from_slice(b"Test\0x\0y\0");

        blob
    }

    #[test]
    fn test_dynamic_container_creation() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8]; // One 8-byte struct
        let blob = create_test_blob();

        let container = DynamicContainer::from_raw(data, &blob).unwrap();
        assert_eq!(container.len(), 1);
        assert_eq!(container.type_name().unwrap(), "Test");
    }

    #[test]
    fn test_field_access() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let blob = create_test_blob();

        let mut container = DynamicContainer::from_raw(data, &blob).unwrap();

        // Read field x (first 4 bytes as u32, little-endian)
        let x: &u32 = container.field(0, "x").unwrap();
        assert_eq!(*x, 0x04030201);

        // Modify field y
        //container.field_mut(0, "y").set(0xDEADBEEF);
        container.field_mut(0, "y").set(0xDEADBEEF_u32);

        let y: &u32 = container.field(0, "y").unwrap();
        assert_eq!(*y, 0xDEADBEEF);
    }
}
