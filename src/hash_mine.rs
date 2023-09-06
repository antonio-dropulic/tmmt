use crate::mine::{Block, Mine, MineError};

use std::{collections::VecDeque, hash::Hash, ops::Add};

use multiset::HashMultiSet;

/// Concrete implementation of [Mine]
/// # Performance
/// - The size of [Mine] scales with O(VALIDATION_WINDOW_SIZE<sup>2</sup>).
///
/// [VALIDATION_WINDOW_SIZE]: Mine<VALIDATION_WINDOW_SIZE>
#[derive(Clone, Debug)]
pub struct HashMine<const VALIDATION_WINDOW_SIZE: usize, B: Block + Hash + Copy> {
    /// Holds [VALIDATION_WINDOW_SIZE] blocks used for validation.
    validation_blocks: VecDeque<B>,
    /// Holds all the possible two element sums from the [validation_blocks](Self::validation_blocks).
    /// Used for quick validation of new blocks.
    block_pair_sums: HashMultiSet<B>,
    /// Used for tracking how many blocks have been validated
    total_blocks: usize,
}

impl<const VALIDATION_WINDOW_SIZE: usize, B> Mine<VALIDATION_WINDOW_SIZE, B>
    for HashMine<VALIDATION_WINDOW_SIZE, B>
where
    B: Block + Hash + Copy,
    for<'a> &'a B: Add<&'a B, Output = B>,
    for<'a> B: Add<&'a B, Output = B>,
{
    /// Create a new mine with given `initialization_blocks`.
    /// No validation is performed on the initialization blocks.
    /// # Performance
    /// This is a potentially costly operation with the running time of O(VALIDATION_WINDOW_SIZE<sup>2</sup>).
    fn new(initialization_blocks: [B; VALIDATION_WINDOW_SIZE]) -> Self {
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

    fn try_extend_one(&mut self, new_block: B) -> Result<(), MineError<VALIDATION_WINDOW_SIZE, B>> {
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

    use crate::mine::Mine as MineTrait;
    use crate::mine::MineError;

    // max size of values in the test file
    type Block = u128;
    type Mine<const V: usize, B> = HashMine<V, B>;

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
