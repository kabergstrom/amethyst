use super::properties::EncodingValue;

/// Trait that defines the encoding buffer writing stragety for a specified
/// shader layout.
/// Every encoder must push exactly one value per iterated entity to the buffer.
///
/// The encoding scheduler is free to implement it in any way that is appropriate
/// for given situation. For example, multiple `EncodeBuffer` views might use
/// the same underlying buffer, but write with a common stride and different offsets.
pub trait EncodeBuffer<T: EncodingValue> {
    /// Push encoded values to the buffer. Must be called exactly once for every entry
    /// in the provided encoding iterator.
    fn push(&mut self, data: T::Value);
}
