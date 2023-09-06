use std::{collections::VecDeque, hash::Hash, iter::Fuse, ops::Add};

use multiset::HashMultiSet;
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
/// # Performance
/// - The size of [Mine] scales with O(VALIDATION_WINDOW_SIZE<sup>2</sup>).
///
/// [VALIDATION_WINDOW_SIZE]: Mine<VALIDATION_WINDOW_SIZE>
#[derive(Clone, Debug)]
pub struct Mine<const VALIDATION_WINDOW_SIZE: usize, B: Block + Hash + Copy> {
    /// Holds [VALIDATION_WINDOW_SIZE] blocks used for validation.
    validation_blocks: VecDeque<B>,
    /// Holds all the possible two element sums from the [validation_blocks](Self::validation_blocks).
    /// Used for quick validation of new blocks.
    block_pair_sums: HashMultiSet<B>,
    /// Used for tracking how many blocks have been validated
    total_blocks: usize,
}

impl<const VALIDATION_WINDOW_SIZE: usize, B> Mine<VALIDATION_WINDOW_SIZE, B>
where
    B: Block + Hash + Copy,
    for<'a> &'a B: Add<&'a B, Output = B>,
    for<'a> B: Add<&'a B, Output = B>,
{
    /// Create a new mine with given `initialization_blocks`.
    /// No validation is performed on the initialization blocks.
    ///
    /// # Performance
    /// This is a potentially costly operation with the running time of O(VALIDATION_WINDOW_SIZE<sup>2</sup>).
    pub fn new(initialization_blocks: [B; VALIDATION_WINDOW_SIZE]) -> Self {
        // Allocating half the max size. Worst case scenario with no overlapping sums
        // requires only 1 more allocation.
        let capacity = VALIDATION_WINDOW_SIZE.pow(2) / 2;
        let mut sums = HashMultiSet::with_capacity(capacity);

        for (i, first) in initialization_blocks[0..VALIDATION_WINDOW_SIZE - 1]
            .iter()
            .enumerate()
        {
            for second in initialization_blocks.iter().skip(i + 1) {
                sums.insert(first + second);
            }
        }

        Self {
            validation_blocks: VecDeque::from(initialization_blocks),
            block_pair_sums: sums,
            total_blocks: VALIDATION_WINDOW_SIZE,
        }
    }

    /// Try to extend the [Mine] by a single [Block] `new_block`.
    /// If the validation is successful the mine is extended, otherwise
    /// a an error [MineError::InvalidBlock] is returned. Details on validation
    /// can be seen in [Mine] documentation.
    ///
    /// If you want to try and add many blocks see [Mine::run].
    pub fn try_extend_one(
        &mut self,
        new_block: B,
    ) -> Result<(), MineError<VALIDATION_WINDOW_SIZE, B>> {
        if !self.block_pair_sums.contains(&new_block) {
            Err(MineError::InvalidBlock(new_block, self.total_blocks + 1))
        } else {
            // New block value is already validated. It is now correct
            // to remove any previous entry and sum entry.

            let old_block = self
                .validation_blocks
                .pop_front()
                .expect("Mine always has VALIDATION_WINDOW_SIZE blocks");

            for block in self.validation_blocks.iter() {
                // remove all sums where the first block was a summand
                self.block_pair_sums.remove(&(old_block + block));
                // add new sums where the new block is a summand
                self.block_pair_sums.insert(new_block + block);
            }

            self.validation_blocks.push_back(new_block);
            self.total_blocks += 1;

            Ok(())
        }
    }

    /// Get the underlying validation blocks window
    pub fn validation_blocks(&self) -> &VecDeque<B> {
        &self.validation_blocks
    }
}

// implementations written in terms of the core functionality [Mine::new] and [Mine::try_extend_one]
impl<const VALIDATION_WINDOW_SIZE: usize, B> Mine<VALIDATION_WINDOW_SIZE, B>
where
    B: Block + Hash + Copy,
    for<'a> &'a B: Add<&'a B, Output = B>,
    for<'a> B: Add<&'a B, Output = B>,
{
    /// Same as [Mine::new] except if the initialization blocks fail to convert
    /// to the desired array [MineError::InvalidInitializationBlocksSize]
    /// is returned.
    pub fn try_new(
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
    pub fn try_extend(
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
    pub fn try_create_and_extend(
        blocks: impl IntoIterator<Item = B>,
    ) -> Result<(), MineError<VALIDATION_WINDOW_SIZE, B>> {
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::{
        array,
        fs::File,
        io::{BufRead, BufReader},
    };

    use super::*;

    // max size of values in the test file
    type Block = u128;

    #[test]
    fn smoke() {
        // 4 initial values
        let initial_blocks = [4, 4, 2, 2];
        let mut mine = Mine::new(initial_blocks);
        assert_eq!(mine.validation_blocks, [4, 4, 2, 2]);

        // inserting 5th
        assert_eq!(mine.try_extend_one(8), Ok(()));
        assert_eq!(
            mine.validation_blocks,
            [4, 2, 2, 8],
            "Unexpected changed validation blocks {:#?}",
            mine.validation_blocks
        );

        // inserting 6th
        assert_eq!(mine.try_extend_one(4), Ok(()));
        assert_eq!(
            mine.validation_blocks,
            [2, 2, 8, 4],
            "Unexpected changed validation blocks {:#?}",
            mine.validation_blocks
        );

        // failing on 7th
        assert_eq!(
            mine.try_extend_one(2),
            Err(MineError::InvalidBlock(2, 7)),
            "Block values present in mine are not necessarily valid sums"
        );
        assert_eq!(
            mine.validation_blocks,
            [2, 2, 8, 4],
            "Expected validation blocks to remain unchanged {:#?}",
            mine.validation_blocks
        );

        // failing on 7th
        assert_eq!(
            mine.try_extend_one(0),
            Err(MineError::InvalidBlock(0, 7)),
            "Sanity checking uint edge cases"
        );
        assert_eq!(
            mine.validation_blocks,
            [2, 2, 8, 4],
            "Expected validation blocks to remain unchanged {:#?}",
            mine.validation_blocks
        );

        // Mine with many same values
        let initial_blocks = [2, 2, 2, 2];
        let mut mine = Mine::new(initial_blocks);
        assert_eq!(mine.validation_blocks, [2, 2, 2, 2]);

        assert_eq!(
            mine.try_extend_one(6),
            Err(MineError::InvalidBlock(6, 5)),
            "Only sums of existing pairs are valid"
        );

        assert_eq!(
            mine.try_extend_one(8),
            Err(MineError::InvalidBlock(8, 5)),
            "Only sums of existing pairs are valid"
        );

        assert_eq!(mine.try_extend_one(4), Ok(()));
        assert_eq!(
            mine.validation_blocks,
            [2, 2, 2, 4],
            "Unexpected changed validation blocks {:#?}",
            mine.validation_blocks
        );
    }

    #[test]
    fn smoke2() {
        let initial_blocks: [Block; 100] = array::from_fn(|i| i as Block + 1);
        let mut mine = Mine::new(initial_blocks);
        let test_blocks: [Block; 99] = array::from_fn(|i| 2 * (i as Block + 1) + 1);

        let result = mine.try_extend(test_blocks);
        assert_eq!(result, Ok(()))
    }

    #[test]
    fn example_with_complex_construction() {
        let initial_blocks = [35, 20, 15, 25, 47];
        let test_blocks = [
            40, 62, 55, 65, 95, 102, 117, 150, 182, 127, 219, 299, 277, 309, 576,
        ];

        let mut mine = Mine::new(initial_blocks);
        let result = mine.try_extend(test_blocks);

        assert_eq!(result, Err(MineError::InvalidBlock(127, 15)));
    }

    #[test]
    fn example_with_simple_construction() {
        let blocks = [
            35, 20, 15, 25, 47, 40, 62, 55, 65, 95, 102, 117, 150, 182, 127, 219, 299, 277, 309,
            576,
        ];

        let result = Mine::<5, u128>::try_create_and_extend(blocks);

        assert_eq!(result, Err(MineError::InvalidBlock(127, 15)));
    }

    #[test]
    fn test_file() {
        let test_file_name = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/challenge_input.txt");
        let test_file = File::open(test_file_name).unwrap();
        let test_file = BufReader::new(test_file);

        let blocks = test_file.lines().map(|block_value| {
            block_value
                .expect("test file must have only valid UTF-8 strings")
                .trim()
                .parse::<u128>()
                .expect("test file must have only valid u128 values")
        });

        let result = Mine::<100, u128>::try_create_and_extend(blocks);

        assert_eq!(result, Err(MineError::InvalidBlock(14, 315)));
    }
}
