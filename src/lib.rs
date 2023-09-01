use multiset::HashMultiSet;
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MineError<const VALIDATION_WINDOW_SIZE: usize> {
    #[error(
        "Initialization blocks must have at least: {} blocks. Size of the blocks provided: {0}",
        VALIDATION_WINDOW_SIZE
    )]
    InvalidInitializationBlocksSize(usize),
    #[error(
        "Validation for block number {1} failed. Invalid block value: {0}.
        A block is valid iff it is the sum of any two blocks in the previous: {}.",
        VALIDATION_WINDOW_SIZE
    )]
    InvalidBlock(Block, usize),
}

// TODO: Wrapper types
// TODO: document addition
pub type Block = u32;
pub type BlockSum = u32;

/// Responsible for mining new [Blocks](Block).
/// A new block is valid [iff](https://en.wikipedia.org/wiki/If_and_only_if) it is the
/// sum of any two blocks in the previous [VALIDATION_WINDOW_SIZE] blocks.
///
/// # Warning
/// - Mine uses unchecked addition of blocks to perform validation.
/// - The size of [Mine] scales with O(VALIDATION_WINDOW_SIZE<sup>2</sup>).
///
/// [VALIDATION_WINDOW_SIZE]: Mine<VALIDATION_WINDOW_SIZE>
#[derive(Clone, Debug)]
pub struct Mine<const VALIDATION_WINDOW_SIZE: usize> {
    /// Holds [VALIDATION_WINDOW_SIZE] blocks used for validation.
    validation_blocks: VecDeque<Block>,
    // TODO: try a different hash function - e.g. FxHasher
    /// Holds all the possible two element sums from the [validation_blocks](Self::validation_blocks).
    /// Used for quick validation of new blocks.
    block_pair_sums: HashMultiSet<BlockSum>,
    /// Used for tracking how many blocks have been validated
    total_blocks: usize,
}

impl<const VALIDATION_WINDOW_SIZE: usize> Mine<VALIDATION_WINDOW_SIZE> {
    /// Create a new mine with given `initialization_blocks`.
    /// No validation is performed on the initialization blocks.
    ///
    /// # Warning
    /// This is a potentially costly operation with the running time of O(VALIDATION_WINDOW_SIZE<sup>2</sup>)
    pub fn new(initialization_blocks: [Block; VALIDATION_WINDOW_SIZE]) -> Self {
        // TODO: THIS MAY ALLOCATE A LOT. NEED TO EXPOSE WITH CAPACITY METHOD.
        let mut sums = HashMultiSet::new();

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
    // TODO: example
    // TODO: consider adding note on performance
    pub fn try_add_block(
        &mut self,
        new_block: Block,
    ) -> Result<(), MineError<VALIDATION_WINDOW_SIZE>> {
        if !self.block_pair_sums.contains(&new_block) {
            Err(MineError::InvalidBlock(new_block, self.total_blocks + 1))
        } else {
            // New block value is already validated. It is now correct
            // to remove any previous entry and sum entry.

            let old_block = self
                .validation_blocks
                .pop_front()
                .expect("Mine always has MINE LENGTH blocks");

            for block in self.validation_blocks.iter() {
                self.block_pair_sums.remove(&(old_block + block));
                self.block_pair_sums.insert(new_block + block);
            }

            self.validation_blocks.push_back(new_block);
            self.total_blocks += 1;

            Ok(())
        }
    }

    /// Try to extend the [Mine] with all the items from the
    /// `blocks` iterator. The method is successful if all
    /// the blocks are successfully added, or the iterator is empty. Otherwise the
    /// error [MineError::InvalidBlock] of the first invalid block
    /// is returned. **IMPORTANT:** Blocks prior to the invalid block are still added
    /// to the mine.
    pub fn run(
        &mut self,
        blocks: impl IntoIterator<Item = Block>,
    ) -> Result<(), MineError<VALIDATION_WINDOW_SIZE>> {
        for block in blocks {
            self.try_add_block(block)?
        }
        Ok(())
    }

    // TODO: rename
    pub fn new_and_run(
        blocks: impl IntoIterator<Item = Block>,
    ) -> Result<(), MineError<VALIDATION_WINDOW_SIZE>> {
        let mut remaining_blocks = blocks.into_iter();
        let initialization_blocks =
            take_with_remainder(&mut remaining_blocks, VALIDATION_WINDOW_SIZE);

        let initialization_blocks =
            initialization_blocks
                .try_into()
                .map_err(|blocks: Vec<Block>| {
                    MineError::InvalidInitializationBlocksSize(blocks.len())
                })?;

        let mut mine = Self::new(initialization_blocks);

        mine.run(remaining_blocks)
    }
}

/// Take n items from the iterator, or less if the iterator has less items.
/// Return the taken items in a Vec. If the iterator was empty an empty vector is returned.
fn take_with_remainder<T, I: Iterator<Item = T>>(iter: &mut I, n: usize) -> Vec<T> {
    let mut counter = 0;
    let mut taken = Vec::with_capacity(n);

    // TODO: [let chains unstable](https://github.com/rust-lang/rfcs/pull/2497)
    while counter < n {
        if let Some(item) = iter.next() {
            counter += 1;
            taken.push(item)
        }
    }

    taken
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use super::*;

    #[test]
    fn smoke() {
        let initial_blocks = [4, 4, 2, 2];
        let mine = Mine::new(initial_blocks);

        // Valid next item
        assert_eq!(mine.clone().try_add_block(8), Ok(()));
        assert_eq!(mine.clone().try_add_block(4), Ok(()));

        // Invalid but present single items
        assert_eq!(
            mine.clone().try_add_block(2),
            Err(MineError::InvalidBlock(2, 5))
        );

        // Invalid random values
        assert_eq!(
            mine.clone().try_add_block(0),
            Err(MineError::InvalidBlock(0, 5))
        );
        assert_eq!(
            mine.clone().try_add_block(1),
            Err(MineError::InvalidBlock(1, 5))
        );

        // cycle valid values
        let mut mine_cycle = mine.clone();

        // first 4 is cycled out, but a new one is cycled in
        assert_eq!(mine_cycle.try_add_block(4), Ok(()));
        // another 4 is cycled out, still one left
        assert_eq!(mine_cycle.try_add_block(8), Ok(()));
        assert_eq!(
            mine_cycle.try_add_block(8),
            Err(MineError::InvalidBlock(8, 7))
        );
    }

    #[test]
    fn example_with_complex_construction() {
        let initial_blocks = [35, 20, 15, 25, 47];
        let test_blocks = [
            40, 62, 55, 65, 95, 102, 117, 150, 182, 127, 219, 299, 277, 309, 576,
        ];

        let mut mine = Mine::new(initial_blocks);
        let result = mine.run(test_blocks);

        assert_eq!(result, Err(MineError::InvalidBlock(127, 15)));
    }

    #[test]
    fn example_with_simple_construction() {
        let blocks = [
            35, 20, 15, 25, 47, 40, 62, 55, 65, 95, 102, 117, 150, 182, 127, 219, 299, 277, 309,
            576,
        ];

        let result = Mine::<5>::new_and_run(blocks);

        assert!(matches!(result, Err(MineError::InvalidBlock(127, 15))));
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
                .parse::<u32>()
                .expect("test file must have only valid u32 values")
        });
        let result = Mine::<100>::new_and_run(blocks);

        // TODO: not sure if this is correct
        assert_eq!(result, Err(MineError::InvalidBlock(14, 315)));
    }
}
