use crate::mine::{Block, Mine};

use std::{collections::VecDeque, ops::Add};

/// Concrete implementation of [Mine]
#[derive(Clone, Debug)]
pub struct TwoPtrMine<const VALIDATION_WINDOW_SIZE: usize, B: Block + Copy + Ord> {
    /// Holds [VALIDATION_WINDOW_SIZE] blocks used for validation.
    validation_blocks: VecDeque<B>,
    /// Holds all the possible two element sums from the [validation_blocks](Self::validation_blocks).
    /// Used for quick validation of new blocks.
    ordered_validation_blocks: Vec<B>,
    /// Tracks how many blocks have been validated
    total_blocks: usize,
}

impl<const VALIDATION_WINDOW_SIZE: usize, B> Mine<VALIDATION_WINDOW_SIZE, B>
    for TwoPtrMine<VALIDATION_WINDOW_SIZE, B>
where
    B: Block + Copy + Ord,
    for<'a> &'a B: Add<&'a B, Output = B>,
    for<'a> B: Add<&'a B, Output = B>,
{
    fn new(mut initialization_blocks: [B; VALIDATION_WINDOW_SIZE]) -> Self {
        let validation_blocks = initialization_blocks;
        initialization_blocks.sort_unstable();

        Self {
            validation_blocks: VecDeque::from(validation_blocks),
            ordered_validation_blocks: Vec::from(initialization_blocks),
            total_blocks: validation_blocks.len(),
        }
    }

    fn try_extend_one(
        &mut self,
        new_block: B,
    ) -> Result<(), crate::mine::MineError<VALIDATION_WINDOW_SIZE, B>> {
        // CHECK NEW BLOCK VALIDITY

        let mut min_to_max = self.ordered_validation_blocks.iter().enumerate();
        let mut max_to_min = self.ordered_validation_blocks.iter().enumerate().rev();

        let mut min_item = min_to_max.next();
        let mut max_item = max_to_min.next();

        while let (Some((i, min)), Some((j, max))) = (min_item, max_item) {
            // all possible (min, max) pairs exhausted
            if i == j {
                return Err(crate::mine::MineError::InvalidBlock(
                    new_block,
                    self.total_blocks + 1,
                ));
            }
            match (min + max).cmp(&new_block) {
                // min element can't be a part of the solution pair
                std::cmp::Ordering::Less => min_item = min_to_max.next(),
                // found solution pair
                std::cmp::Ordering::Equal => break,
                // max element can't be a part of the solution pair
                std::cmp::Ordering::Greater => max_item = max_to_min.next(),
            }

            // TODO: we can search for the old block in this loop as an optimization attempt
        }

        // NEW BLOCK IS VALID
        // now we can safely remove/insert items to validation blocks

        let old_block = self
            .validation_blocks
            .pop_front()
            .expect("validation_blocks have a minimum size VALIDATION_WINDOW_SIZE");
        self.validation_blocks.push_back(new_block);

        // TODO:
        // - try mapping validation blocks to ordered validation blocks when you perform sort
        // - try linear search, for small enough windows / block sizes it may be faster
        let old_block_idx = self
            .ordered_validation_blocks
            .binary_search(&old_block)
            .unwrap_or_else(|i| i);
        self.ordered_validation_blocks.remove(old_block_idx);

        let new_block_idx = self
            .ordered_validation_blocks
            .binary_search(&new_block)
            .unwrap_or_else(|i| i);
        self.ordered_validation_blocks
            .insert(new_block_idx, new_block);

        self.total_blocks += 1;

        Ok(())
    }
}

// TODO: macro for tests
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
    type Mine<const V: usize, B> = TwoPtrMine<V, B>;

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
