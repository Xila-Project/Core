//! Type-Length-Value (TLV) encoding and decoding.
//!
//! This module provides a no-allocation, idiomatic Rust interface for working with TLV-encoded data.
//! TLV is a simple encoding scheme where each element consists of:
//! - Type: A tag identifying the data type (u16)
//! - Length: The length of the value in bytes (u16)
//! - Value: The actual data bytes
//!
//! # Examples
//!
//! ```
//! use shared::tlv::{TlvEncoder, TlvDecoder};
//!
//! // Encoding with a fixed buffer
//! let mut buffer = [0u8; 128];
//! let mut encoder = TlvEncoder::new(&mut buffer);
//! encoder.add_bytes(1, b"hello").unwrap();
//! encoder.add_bytes(2, b"world").unwrap();
//! let encoded = encoder.as_slice();
//!
//! // Decoding
//! let decoder = TlvDecoder::new(encoded);
//! for entry in decoder {
//!     let (tag, value) = entry.unwrap();
//!     println!("Tag: {}, Value: {:?}", tag, value);
//! }
//! ```

use core::fmt;

#[cfg(test)]
extern crate alloc;

/// Error types for TLV operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlvError {
    /// Not enough data to read the complete TLV structure
    InsufficientData,
    /// Invalid TLV format
    InvalidFormat,
    /// Tag not found
    TagNotFound,
    /// Buffer is full, cannot add more entries
    BufferFull,
    /// Value is too large to encode
    ValueTooLarge,
}

impl fmt::Display for TlvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlvError::InsufficientData => write!(f, "Insufficient data in TLV buffer"),
            TlvError::InvalidFormat => write!(f, "Invalid TLV format"),
            TlvError::TagNotFound => write!(f, "Tag not found in TLV data"),
            TlvError::BufferFull => write!(f, "TLV buffer is full"),
            TlvError::ValueTooLarge => write!(f, "Value is too large to encode"),
        }
    }
}

/// A TLV encoder for building TLV-encoded data in a fixed buffer.
///
/// # Examples
///
/// ```
/// use shared::tlv::TlvEncoder;
///
/// let mut buffer = [0u8; 128];
/// let mut encoder = TlvEncoder::new(&mut buffer);
/// encoder.add_bytes(1, b"hello").unwrap();
/// encoder.add_bytes(2, &[1, 2, 3, 4]).unwrap();
/// let data = encoder.as_slice();
/// ```
#[derive(Debug)]
pub struct TlvEncoder<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> TlvEncoder<'a> {
    /// Creates a new TLV encoder with the given buffer.
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn add_bytes_iterator<I>(&mut self, tag: u16, values: I) -> Result<&mut Self, TlvError>
    where
        I: Iterator,
        I::Item: AsRef<[u8]>,
    {
        // Write tag placeholder
        let tag_position = self.position;
        if self.position + 4 > self.buffer.len() {
            return Err(TlvError::BufferFull);
        }

        self.buffer[self.position..self.position + 2].copy_from_slice(&tag.to_be_bytes());
        self.position += 2;

        // Reserve space for length (will be filled later)
        let length_position = self.position;
        self.position += 2;

        // Write values and track total length
        let start_position = self.position;
        for value in values {
            let value_bytes = value.as_ref();

            if self.position + value_bytes.len() > self.buffer.len() {
                // Rollback on error
                self.position = tag_position;
                return Err(TlvError::BufferFull);
            }

            self.buffer[self.position..self.position + value_bytes.len()]
                .copy_from_slice(value_bytes);
            self.position += value_bytes.len();
        }

        let total_length = self.position - start_position;

        if total_length > u16::MAX as usize {
            // Rollback on error
            self.position = tag_position;
            return Err(TlvError::ValueTooLarge);
        }

        // Write the actual length
        self.buffer[length_position..length_position + 2]
            .copy_from_slice(&(total_length as u16).to_be_bytes());

        Ok(self)
    }

    /// Adds a TLV entry with the given tag and value.
    ///
    /// Returns an error if the value is too large or the buffer is full.
    pub fn add_bytes(&mut self, tag: u16, value: &[u8]) -> Result<&mut Self, TlvError> {
        if value.len() > u16::MAX as usize {
            return Err(TlvError::ValueTooLarge);
        }

        let required_space = 4 + value.len(); // 2 bytes tag + 2 bytes length + value
        if self.position + required_space > self.buffer.len() {
            return Err(TlvError::BufferFull);
        }

        // Write tag
        self.buffer[self.position..self.position + 2].copy_from_slice(&tag.to_be_bytes());
        self.position += 2;

        // Write length
        self.buffer[self.position..self.position + 2]
            .copy_from_slice(&(value.len() as u16).to_be_bytes());
        self.position += 2;

        // Write value
        self.buffer[self.position..self.position + value.len()].copy_from_slice(value);
        self.position += value.len();

        Ok(self)
    }

    /// Adds a TLV entry with a string value.
    pub fn add_str(&mut self, tag: u16, value: &str) -> Result<&mut Self, TlvError> {
        self.add_bytes(tag, value.as_bytes())
    }

    pub fn add_u8(&mut self, tag: u16, value: u8) -> Result<&mut Self, TlvError> {
        self.add_bytes(tag, &[value])
    }

    pub fn add_u16(&mut self, tag: u16, value: u16) -> Result<&mut Self, TlvError> {
        self.add_bytes(tag, &value.to_be_bytes())
    }

    /// Adds a TLV entry with a u32 value.
    pub fn add_u32(&mut self, tag: u16, value: u32) -> Result<&mut Self, TlvError> {
        self.add_bytes(tag, &value.to_be_bytes())
    }

    /// Adds a TLV entry with a u64 value.
    pub fn add_u64(&mut self, tag: u16, value: u64) -> Result<&mut Self, TlvError> {
        self.add_bytes(tag, &value.to_be_bytes())
    }

    /// Returns the number of bytes written to the buffer.
    pub fn len(&self) -> usize {
        self.position
    }

    /// Returns `true` if no data has been written.
    pub fn is_empty(&self) -> bool {
        self.position == 0
    }

    /// Returns the remaining space in the buffer.
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    /// Returns a reference to the encoded data.
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.position]
    }

    /// Clears the encoder, resetting the position to the beginning.
    pub fn clear(&mut self) {
        self.position = 0;
    }

    /// Expands the last/top container in the buffer by appending data to it and updating its length field.
    ///
    /// This is useful when encoding nested TLV structures where you add a container
    /// entry with an initial length, then want to append child entries to it, and
    /// automatically update the container's length to reflect the actual size of its contents.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to append to the last container's value
    ///
    /// # Returns
    ///
    /// Returns an error if:
    /// - The buffer is empty
    /// - The new length would exceed u16::MAX
    /// - There's not enough space in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use shared::tlv::TlvEncoder;
    ///
    /// let mut buffer = [0u8; 128];
    /// let mut encoder = TlvEncoder::new(&mut buffer);
    ///
    /// // Add a container with initial empty value
    /// encoder.add_bytes(1, &[]).unwrap();
    ///
    /// // Add nested entries by expanding the container
    /// encoder.expand_last_container(b"hello").unwrap();
    /// encoder.expand_last_container(b"world").unwrap();
    /// ```
    pub fn expand_last_container(&mut self, data: &[u8]) -> Result<&mut Self, TlvError> {
        if self.position < 4 {
            return Err(TlvError::InvalidFormat);
        }

        // Find the start of the last TLV entry by scanning from the beginning
        let pos = self.position;
        let mut last_entry_start = None;
        let mut last_entry_length = 0;

        // Parse from the beginning to find all entries
        let mut scan_pos = 0;
        while scan_pos < pos {
            if scan_pos + 4 > pos {
                return Err(TlvError::InvalidFormat);
            }

            let entry_start = scan_pos;
            scan_pos += 2; // Skip tag

            let length =
                u16::from_be_bytes([self.buffer[scan_pos], self.buffer[scan_pos + 1]]) as usize;
            scan_pos += 2; // Skip length field
            scan_pos += length; // Skip value

            if scan_pos <= pos {
                last_entry_start = Some(entry_start);
                last_entry_length = length;
            }
        }

        let entry_start = last_entry_start.ok_or(TlvError::InvalidFormat)?;

        // Calculate new length
        let new_length = last_entry_length
            .checked_add(data.len())
            .ok_or(TlvError::ValueTooLarge)?;

        if new_length > u16::MAX as usize {
            return Err(TlvError::ValueTooLarge);
        }

        // Check if we have enough space to append the data
        if self.position + data.len() > self.buffer.len() {
            return Err(TlvError::BufferFull);
        }

        // Append the data to the end of the buffer
        self.buffer[self.position..self.position + data.len()].copy_from_slice(data);
        self.position += data.len();

        // Update the length field (at offset 2 from entry start)
        let length_offset = entry_start + 2;
        self.buffer[length_offset..length_offset + 2]
            .copy_from_slice(&(new_length as u16).to_be_bytes());

        Ok(self)
    }
}

/// A TLV decoder for parsing TLV-encoded data.
///
/// # Examples
///
/// ```
/// use shared::tlv::{TlvEncoder, TlvDecoder};
///
/// let mut buffer = [0u8; 128];
/// let mut encoder = TlvEncoder::new(&mut buffer);
/// encoder.add_bytes(1, b"hello").unwrap();
/// let encoded = encoder.as_slice();
///
/// let decoder = TlvDecoder::new(encoded);
/// for entry in decoder {
///     let (tag, value) = entry.unwrap();
///     println!("Tag: {}, Value: {:?}", tag, value);
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TlvDecoder<'a> {
    data: &'a [u8],
}

impl<'a> TlvDecoder<'a> {
    /// Creates a new TLV decoder from the given data.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Returns an iterator over all TLV entries.
    pub fn iter(&self) -> TlvIterator<'a> {
        TlvIterator {
            data: self.data,
            position: 0,
        }
    }

    /// Finds the first entry with the given tag.
    pub fn find_bytes(&self, tag: u16) -> Result<&'a [u8], TlvError> {
        for entry in self.iter() {
            let (entry_tag, value) = entry?;
            if entry_tag == tag {
                return Ok(value);
            }
        }
        Err(TlvError::TagNotFound)
    }

    /// Finds the first entry with the given tag and parses it as a string.
    pub fn find_str(&self, tag: u16) -> Result<&'a str, TlvError> {
        let value = self.find_bytes(tag)?;
        core::str::from_utf8(value).map_err(|_| TlvError::InvalidFormat)
    }

    /// Finds the first entry with the given tag and parses it as a u32.
    pub fn find_u32(&self, tag: u16) -> Result<u32, TlvError> {
        let value = self.find_bytes(tag)?;
        if value.len() != 4 {
            return Err(TlvError::InvalidFormat);
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(value);
        Ok(u32::from_be_bytes(bytes))
    }

    /// Finds the first entry with the given tag and parses it as a u64.
    pub fn find_u64(&self, tag: u16) -> Result<u64, TlvError> {
        let value = self.find_bytes(tag)?;
        if value.len() != 8 {
            return Err(TlvError::InvalidFormat);
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(value);
        Ok(u64::from_be_bytes(bytes))
    }

    /// Returns an iterator over all entries with the given tag.
    pub fn find_all(&self, tag: u16) -> impl Iterator<Item = &'a [u8]> + 'a {
        self.iter()
            .filter_map(move |entry| entry.ok())
            .filter(move |(entry_tag, _)| *entry_tag == tag)
            .map(|(_, value)| value)
    }

    /// Returns the number of TLV entries in the data.
    pub fn count(&self) -> usize {
        self.iter().count()
    }

    /// Validates that the TLV data is well-formed.
    pub fn validate(&self) -> Result<(), TlvError> {
        for entry in self.iter() {
            entry?;
        }
        Ok(())
    }
}

impl<'a> IntoIterator for TlvDecoder<'a> {
    type Item = Result<(u16, &'a [u8]), TlvError>;
    type IntoIter = TlvIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over TLV entries.
#[derive(Debug, Clone)]
pub struct TlvIterator<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Iterator for TlvIterator<'a> {
    type Item = Result<(u16, &'a [u8]), TlvError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.data.len() {
            return None;
        }

        // Check if we have enough data for tag and length (4 bytes)
        if self.position + 4 > self.data.len() {
            return Some(Err(TlvError::InsufficientData));
        }

        // Read tag (2 bytes)
        let tag = u16::from_be_bytes([self.data[self.position], self.data[self.position + 1]]);
        self.position += 2;

        // Read length (2 bytes)
        let length =
            u16::from_be_bytes([self.data[self.position], self.data[self.position + 1]]) as usize;
        self.position += 2;

        // Check if we have enough data for the value
        if self.position + length > self.data.len() {
            return Some(Err(TlvError::InsufficientData));
        }

        // Read value
        let value = &self.data[self.position..self.position + length];
        self.position += length;

        Some(Ok((tag, value)))
    }
}

impl<'a> TlvIterator<'a> {
    /// Returns the current position in the data buffer.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns the remaining bytes in the buffer.
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }
}

/// A key-value map view of TLV data.
///
/// This provides a map-like interface for accessing TLV entries by tag.
/// Note that if multiple entries have the same tag, only the first one is accessible.
///
/// # Examples
///
/// ```
/// use shared::tlv::{TlvEncoder, TlvMap};
///
/// let mut buffer = [0u8; 128];
/// let mut encoder = TlvEncoder::new(&mut buffer);
/// encoder.add_bytes(1, b"hello").unwrap();
/// encoder.add_bytes(2, b"world").unwrap();
/// let encoded = encoder.as_slice();
///
/// let map = TlvMap::new(encoded);
/// assert_eq!(map.get(1).unwrap(), b"hello");
/// assert_eq!(map.get(2).unwrap(), b"world");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TlvMap<'a> {
    decoder: TlvDecoder<'a>,
}

impl<'a> TlvMap<'a> {
    /// Creates a new TLV map from the given data.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            decoder: TlvDecoder::new(data),
        }
    }

    /// Gets the value associated with the given tag.
    pub fn get(&self, tag: u16) -> Option<&'a [u8]> {
        self.decoder.find_bytes(tag).ok()
    }

    /// Gets the value associated with the given tag as a string.
    pub fn get_str(&self, tag: u16) -> Option<&'a str> {
        self.decoder.find_str(tag).ok()
    }

    /// Gets the value associated with the given tag as a u32.
    pub fn get_u32(&self, tag: u16) -> Option<u32> {
        self.decoder.find_u32(tag).ok()
    }

    /// Gets the value associated with the given tag as a u64.
    pub fn get_u64(&self, tag: u16) -> Option<u64> {
        self.decoder.find_u64(tag).ok()
    }

    /// Returns `true` if the map contains an entry with the given tag.
    pub fn contains_key(&self, tag: u16) -> bool {
        self.get(tag).is_some()
    }

    /// Returns an iterator over all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (u16, &'a [u8])> + 'a {
        self.decoder.iter().filter_map(|entry| entry.ok())
    }

    /// Returns an iterator over all keys (tags).
    pub fn keys(&self) -> impl Iterator<Item = u16> + 'a {
        self.iter().map(|(tag, _)| tag)
    }

    /// Returns an iterator over all values.
    pub fn values(&self) -> impl Iterator<Item = &'a [u8]> + 'a {
        self.iter().map(|(_, value)| value)
    }

    /// Returns the number of entries in the map.
    pub fn len(&self) -> usize {
        self.decoder.count()
    }

    /// Returns `true` if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_encode_decode_basic() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);
        encoder.add_bytes(1, b"hello").unwrap();
        encoder.add_bytes(2, b"world").unwrap();
        let data = encoder.as_slice();

        let decoder = TlvDecoder::new(data);
        let mut iter = decoder.iter();

        let (tag1, value1) = iter.next().unwrap().unwrap();
        assert_eq!(tag1, 1);
        assert_eq!(value1, b"hello");

        let (tag2, value2) = iter.next().unwrap().unwrap();
        assert_eq!(tag2, 2);
        assert_eq!(value2, b"world");

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_encode_decode_empty_value() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);
        encoder.add_bytes(1, b"").unwrap();
        let data = encoder.as_slice();

        let decoder = TlvDecoder::new(data);
        let value = decoder.find_bytes(1).unwrap();
        assert_eq!(value, b"");
    }

    #[test]
    fn test_find_methods() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);
        encoder.add_str(1, "hello").unwrap();
        encoder.add_u32(2, 42).unwrap();
        encoder.add_u64(3, 1234567890).unwrap();
        let data = encoder.as_slice();

        let decoder = TlvDecoder::new(data);
        assert_eq!(decoder.find_str(1).unwrap(), "hello");
        assert_eq!(decoder.find_u32(2).unwrap(), 42);
        assert_eq!(decoder.find_u64(3).unwrap(), 1234567890);
    }

    #[test]
    fn test_find_not_found() {
        let mut buffer = [0u8; 128];
        let encoder = TlvEncoder::new(&mut buffer);
        let data = encoder.as_slice();

        let decoder = TlvDecoder::new(data);
        assert_eq!(decoder.find_bytes(1), Err(TlvError::TagNotFound));
    }

    #[test]
    fn test_find_all() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);
        encoder.add_bytes(1, b"first").unwrap();
        encoder.add_bytes(1, b"second").unwrap();
        encoder.add_bytes(2, b"other").unwrap();
        encoder.add_bytes(1, b"third").unwrap();
        let data = encoder.as_slice();

        let decoder = TlvDecoder::new(data);
        let all_ones: Vec<_> = decoder.find_all(1).collect();
        assert_eq!(all_ones.len(), 3);
        assert_eq!(all_ones[0], b"first");
        assert_eq!(all_ones[1], b"second");
        assert_eq!(all_ones[2], b"third");
    }

    #[test]
    fn test_map_interface() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);
        encoder.add_bytes(1, b"value1").unwrap();
        encoder.add_bytes(2, b"value2").unwrap();
        encoder.add_bytes(3, b"value3").unwrap();
        let data = encoder.as_slice();

        let map = TlvMap::new(data);
        assert_eq!(map.get(1).unwrap(), b"value1");
        assert_eq!(map.get(2).unwrap(), b"value2");
        assert_eq!(map.get(3).unwrap(), b"value3");
        assert!(map.get(4).is_none());
        assert!(map.contains_key(1));
        assert!(!map.contains_key(4));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_map_iterators() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);
        encoder.add_bytes(1, b"a").unwrap();
        encoder.add_bytes(2, b"b").unwrap();
        encoder.add_bytes(3, b"c").unwrap();
        let data = encoder.as_slice();

        let map = TlvMap::new(data);
        let mut keys = map.keys();
        assert_eq!(keys.next(), Some(1));
        assert_eq!(keys.next(), Some(2));
        assert_eq!(keys.next(), Some(3));
        assert_eq!(keys.next(), None);

        let mut values = map.values();
        assert_eq!(values.next(), Some(b"a".as_ref()));
        assert_eq!(values.next(), Some(b"b".as_ref()));
        assert_eq!(values.next(), Some(b"c".as_ref()));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn test_invalid_data() {
        // Incomplete tag
        let data = [0x00];
        let decoder = TlvDecoder::new(&data);
        let mut iter = decoder.iter();
        assert!(matches!(
            iter.next().unwrap(),
            Err(TlvError::InsufficientData)
        ));

        // Incomplete length
        let data = [0x00, 0x01, 0x00];
        let decoder = TlvDecoder::new(&data);
        let mut iter = decoder.iter();
        assert!(matches!(
            iter.next().unwrap(),
            Err(TlvError::InsufficientData)
        ));

        // Incomplete value
        let data = [0x00, 0x01, 0x00, 0x05, 0x01, 0x02];
        let decoder = TlvDecoder::new(&data);
        let mut iter = decoder.iter();
        assert!(matches!(
            iter.next().unwrap(),
            Err(TlvError::InsufficientData)
        ));
    }

    #[test]
    fn test_validate() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);
        encoder.add_bytes(1, b"test").unwrap();
        let data = encoder.as_slice();

        let decoder = TlvDecoder::new(data);
        assert!(decoder.validate().is_ok());

        // Invalid data
        let invalid_data = [0x00, 0x01, 0x00, 0x05, 0x01];
        let decoder = TlvDecoder::new(&invalid_data);
        assert!(decoder.validate().is_err());
    }

    #[test]
    fn test_encoder_chaining() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);
        encoder
            .add_bytes(1, b"a")
            .unwrap()
            .add_bytes(2, b"b")
            .unwrap()
            .add_bytes(3, b"c")
            .unwrap();
        let data = encoder.as_slice();

        let decoder = TlvDecoder::new(data);
        assert_eq!(decoder.count(), 3);
    }

    #[test]
    fn test_iterator_remaining() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);
        encoder.add_bytes(1, b"test").unwrap();
        let data = encoder.as_slice();

        let mut iter = TlvDecoder::new(data).iter();
        assert!(iter.remaining() > 0);
        iter.next();
        assert_eq!(iter.remaining(), 0);
    }

    #[test]
    fn test_buffer_full() {
        let mut buffer = [0u8; 10];
        let mut encoder = TlvEncoder::new(&mut buffer);

        // This should succeed (4 bytes header + 5 bytes value = 9 bytes)
        assert!(encoder.add_bytes(1, b"hello").is_ok());

        // This should fail (only 1 byte remaining, need 4 + value)
        assert!(matches!(
            encoder.add_bytes(2, b"x"),
            Err(TlvError::BufferFull)
        ));
    }

    #[test]
    fn test_value_too_large() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        let large_data = [0u8; (u16::MAX as usize) + 1];
        assert!(matches!(
            encoder.add_bytes(1, &large_data),
            Err(TlvError::ValueTooLarge)
        ));
    }

    #[test]
    fn test_encoder_clear() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        encoder.add_bytes(1, b"test").unwrap();
        assert_eq!(encoder.len(), 8); // 2 (tag) + 2 (length) + 4 (value)

        encoder.clear();
        assert_eq!(encoder.len(), 0);
        assert!(encoder.is_empty());

        encoder.add_bytes(2, b"new").unwrap();
        let decoder = TlvDecoder::new(encoder.as_slice());
        assert_eq!(decoder.find_bytes(2).unwrap(), b"new");
        assert!(decoder.find_bytes(1).is_err());
    }

    #[test]
    fn test_encoder_remaining() {
        let mut buffer = [0u8; 20];
        let mut encoder = TlvEncoder::new(&mut buffer);

        assert_eq!(encoder.remaining(), 20);
        encoder.add_bytes(1, b"test").unwrap(); // 4 + 4 = 8 bytes
        assert_eq!(encoder.remaining(), 12);
    }

    #[test]
    fn test_expand_last_container() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        // Add a container with initial length 0
        encoder.add_bytes(1, &[]).unwrap();

        // Expand the container by adding data
        encoder.expand_last_container(b"hello").unwrap();
        encoder.expand_last_container(b"world").unwrap();

        // Verify the container was expanded correctly
        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);

        let value = decoder.find_bytes(1).unwrap();
        assert_eq!(value, b"helloworld");
    }

    #[test]
    fn test_expand_last_container_multiple_entries() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        // Add first entry
        encoder.add_bytes(1, b"first").unwrap();

        // Add second entry (container)
        encoder.add_bytes(2, b"init").unwrap();

        // Expand the last (second) container
        encoder.expand_last_container(b"ial").unwrap();

        // Verify
        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);

        let value1 = decoder.find_bytes(1).unwrap();
        assert_eq!(value1, b"first");

        let value2 = decoder.find_bytes(2).unwrap();
        assert_eq!(value2, b"initial");
    }

    #[test]
    fn test_expand_last_container_empty_buffer() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        // Should fail on empty buffer
        assert!(matches!(
            encoder.expand_last_container(b"test"),
            Err(TlvError::InvalidFormat)
        ));
    }

    #[test]
    fn test_expand_last_container_nested() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        // Add a container
        encoder.add_bytes(1, &[]).unwrap();

        // Build nested TLV structure by expanding the container
        let mut nested_buffer = [0u8; 64];
        let mut nested_encoder = TlvEncoder::new(&mut nested_buffer);
        nested_encoder.add_bytes(2, b"hello").unwrap();
        nested_encoder.add_bytes(3, b"world").unwrap();

        // Expand container with nested entries
        encoder
            .expand_last_container(nested_encoder.as_slice())
            .unwrap();

        // Verify the nested structure
        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);
        let container_value = decoder.find_bytes(1).unwrap();

        // Decode the nested structure
        let nested_decoder = TlvDecoder::new(container_value);
        assert_eq!(nested_decoder.find_bytes(2).unwrap(), b"hello");
        assert_eq!(nested_decoder.find_bytes(3).unwrap(), b"world");
    }

    #[test]
    fn test_expand_last_container_buffer_full() {
        let mut buffer = [0u8; 10];
        let mut encoder = TlvEncoder::new(&mut buffer);

        // Add a small container (4 bytes header + 1 byte value = 5 bytes)
        encoder.add_bytes(1, b"x").unwrap();

        // Try to expand beyond buffer capacity (only 5 bytes left, need more)
        assert!(matches!(
            encoder.expand_last_container(&[0u8; 10]),
            Err(TlvError::BufferFull)
        ));
    }

    #[test]
    fn test_expand_last_container_value_too_large() {
        use alloc::vec;
        // Use a large buffer to avoid BufferFull error
        let mut buffer = vec![0u8; 70000];
        let mut encoder = TlvEncoder::new(&mut buffer);

        // Add a container with near-maximum length
        let large_data = [0u8; 65530]; // Close to u16::MAX
        encoder.add_bytes(1, &large_data).unwrap();

        // Try to expand beyond u16::MAX
        assert!(matches!(
            encoder.expand_last_container(&[0u8; 10]),
            Err(TlvError::ValueTooLarge)
        ));
    }

    #[test]
    fn test_add_bytes_iterator_basic() {
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        let values = &[b"hello".as_slice(), b"world".as_slice(), b"test".as_slice()];
        encoder.add_bytes_iterator(1, values.into_iter()).unwrap();

        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);
        let value = decoder.find_bytes(1).unwrap();

        // Should concatenate all values
        assert_eq!(value, b"helloworldtest");
    }

    #[test]
    fn test_add_bytes_iterator_empty() {
        use alloc::vec;
        use alloc::vec::Vec;
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        let values: Vec<&[u8]> = vec![];
        encoder.add_bytes_iterator(1, values.into_iter()).unwrap();

        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);
        let value = decoder.find_bytes(1).unwrap();

        assert_eq!(value, b"");
    }

    #[test]
    fn test_add_bytes_iterator_single_value() {
        use alloc::vec;
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        let values = vec![b"single".as_slice()];
        encoder.add_bytes_iterator(1, values.into_iter()).unwrap();

        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);
        let value = decoder.find_bytes(1).unwrap();

        assert_eq!(value, b"single");
    }

    #[test]
    fn test_add_bytes_iterator_with_strings() {
        use alloc::string::String;
        use alloc::vec;
        use alloc::vec::Vec;
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        let values: Vec<String> = vec![
            String::from("foo"),
            String::from("bar"),
            String::from("baz"),
        ];
        encoder
            .add_bytes_iterator(1, values.iter().map(|s| s.as_bytes()))
            .unwrap();

        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);
        let value = decoder.find_bytes(1).unwrap();

        assert_eq!(value, b"foobarbaz");
    }

    #[test]
    fn test_add_bytes_iterator_multiple_entries() {
        use alloc::vec;
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        let values1 = vec![b"hello".as_slice(), b"world".as_slice()];
        encoder.add_bytes_iterator(1, values1.into_iter()).unwrap();

        let values2 = vec![b"foo".as_slice(), b"bar".as_slice()];
        encoder.add_bytes_iterator(2, values2.into_iter()).unwrap();

        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);

        assert_eq!(decoder.find_bytes(1).unwrap(), b"helloworld");
        assert_eq!(decoder.find_bytes(2).unwrap(), b"foobar");
    }

    #[test]
    fn test_add_bytes_iterator_buffer_full() {
        use alloc::vec;
        let mut buffer = [0u8; 10];
        let mut encoder = TlvEncoder::new(&mut buffer);

        let values = vec![b"this".as_slice(), b"is".as_slice(), b"toolong".as_slice()];

        // 4 bytes header + 15 bytes total value = 19 bytes > 10 bytes buffer
        assert!(matches!(
            encoder.add_bytes_iterator(1, values.into_iter()),
            Err(TlvError::BufferFull)
        ));
    }

    #[test]
    fn test_add_bytes_iterator_chaining() {
        use alloc::vec;
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        let values1 = vec![b"a".as_slice(), b"b".as_slice()];
        let values2 = vec![b"c".as_slice(), b"d".as_slice()];

        encoder
            .add_bytes_iterator(1, values1.into_iter())
            .unwrap()
            .add_bytes_iterator(2, values2.into_iter())
            .unwrap();

        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);

        assert_eq!(decoder.find_bytes(1).unwrap(), b"ab");
        assert_eq!(decoder.find_bytes(2).unwrap(), b"cd");
    }

    #[test]
    fn test_add_bytes_iterator_with_vec() {
        use alloc::vec;
        let mut buffer = [0u8; 128];
        let mut encoder = TlvEncoder::new(&mut buffer);

        let values = vec![vec![1, 2, 3], vec![4, 5], vec![6, 7, 8, 9]];
        encoder
            .add_bytes_iterator(1, values.iter().map(|v| v.as_slice()))
            .unwrap();

        let data = encoder.as_slice();
        let decoder = TlvDecoder::new(data);
        let value = decoder.find_bytes(1).unwrap();

        assert_eq!(value, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
