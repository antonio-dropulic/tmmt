use std::{iter::Fuse, ops::Add};

use thiserror::Error;

/// Block in a [Mine]. Has blanket implementation for numerical types.
pub trait Block: Eq + Add<Output = Self> + Sized {}
impl<T> Block for T where T: Eq + Add<Output = Self> + Sized {}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MineError<const VALIDATION_WINDOW_SIZE: usize, B: Block> {
    #[error(
        "Initialization blocks must have at least: {} blocks. Size of the blocks provided: {0}",
        VALIDATION_WINDOW_SIZE
    )]
    InvalidInitializationBlocksSize,
    #[error(
        "Validation for block number {1} failed. Invalid block value: {0}.
        A block is valid iff it is the sum of any two blocks in the previous: {}.",
        VALIDATION_WINDOW_SIZE
    )]
    InvalidBlock(B, usize),
}

/// Responsible for mining new [Blocks](Block).
/// A new block is valid [iff](https://en.wikipedia.org/wiki/If_and_only_if) it's the
/// sum of any two blocks in the previous [VALIDATION_WINDOW_SIZE] blocks.
///
/// [VALIDATION_WINDOW_SIZE]: Mine<VALIDATION_WINDOW_SIZE>s
pub trait Mine<const VALIDATION_WINDOW_SIZE: usize, B: Block> {
    /// Create a new mine with given `initialization_blocks`.
    /// No validation is performed on the initialization blocks.
    fn new(initialization_blocks: [B; VALIDATION_WINDOW_SIZE]) -> Self;
    /// Try to extend the [Mine] with all the items from the
    /// `blocks` iterator. The method is successful if all
    /// the blocks are successfully added, or the iterator is empty. Otherwise the
    /// error [MineError::InvalidBlock] of the first invalid block
    /// is returned. **IMPORTANT:** Blocks prior to the invalid block are still added
    /// to the mine.
    fn try_extend_one(&mut self, new_block: B) -> Result<(), MineError<VALIDATION_WINDOW_SIZE, B>>;

    /// Same as [Mine::new] except if the initialization blocks fail to convert
    /// to the desired array [MineError::InvalidInitializationBlocksSize]
    /// is returned.
    fn try_new(
        initialization_blocks: impl TryInto<[B; VALIDATION_WINDOW_SIZE]>,
    ) -> Result<Self, MineError<VALIDATION_WINDOW_SIZE, B>>
    where
        Self: Sized,
    {
        let initialization_blocks: [B; VALIDATION_WINDOW_SIZE] =
            initialization_blocks.try_into().map_err(|_| {
                MineError::<VALIDATION_WINDOW_SIZE, B>::InvalidInitializationBlocksSize
            })?;

        Ok(Self::new(initialization_blocks))
    }

    /// Try to extend the [Mine] with all the items from the
    /// `blocks` iterator. The method is successful if all
    /// the blocks are successfully added, or the iterator is empty. Otherwise the
    /// error [MineError::InvalidBlock] of the first invalid block
    /// is returned. **IMPORTANT:** Blocks prior to the invalid block are still added
    /// to the mine.
    fn try_extend(
        &mut self,
        blocks: impl IntoIterator<Item = B>,
    ) -> Result<(), MineError<VALIDATION_WINDOW_SIZE, B>> {
        for block in blocks {
            self.try_extend_one(block)?
        }
        Ok(())
    }

    /// Try and create and extend a Mine from a single iterator.
    /// First [VALIDATION_WINDOW_SIZE] elements of `blocks` are used to create the mine.
    /// The remainder of elements are used to try_extend the mine.
    ///
    /// # Errors
    /// - If the `blocks` iterator length is less than [VALIDATION_WINDOW_SIZE] then
    /// [MineError::InvalidInitializationBlocksSize] is returned.
    /// - If any remaining element can't be validated [MineError::InvalidBlock] is returned.
    ///
    /// [VALIDATION_WINDOW_SIZE]: Mine<VALIDATION_WINDOW_SIZE>
    fn try_create_and_extend(
        blocks: impl IntoIterator<Item = B>,
    ) -> Result<(), MineError<VALIDATION_WINDOW_SIZE, B>>
    where
        Self: Sized,
    {
        let (initialization_blocks, remaining_blocks) =
            take_with_remainder(blocks.into_iter(), VALIDATION_WINDOW_SIZE);

        let initialization_blocks = initialization_blocks
            .try_into()
            .map_err(|_| MineError::InvalidInitializationBlocksSize)?;

        let mut mine = Self::new(initialization_blocks);

        mine.try_extend(remaining_blocks)
    }
}

/// Take n items from the iterator, or less if the iterator has less items.
/// Return the taken items in a Vec. If the iterator was empty an empty vector is returned.
fn take_with_remainder<T, I: Iterator<Item = T>>(mut iter: I, n: usize) -> (Vec<T>, Fuse<I>) {
    let mut taken;

    if iter.size_hint().0 < n {
        taken = Vec::new();
    } else {
        taken = Vec::with_capacity(n);
    }

    for _ in 0..n {
        if let Some(item) = iter.next() {
            taken.push(item)
        }
    }

    let remainder = iter.fuse();

    (taken, remainder)
}
