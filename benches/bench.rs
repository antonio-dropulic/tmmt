use std::ptr::copy_nonoverlapping;

use tmmt::*;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;

// size found in test file
type Block = u128;

const fn generate_input_blocks<const SIZE: usize>() -> [Block; SIZE] {
    let initial_blocks = [0; SIZE];

    unsafe {
        let arr_ptr = &initial_blocks as *const [Block];
        let arr_ptr_mut = arr_ptr as *mut [Block] as *mut Block;
        copy_nonoverlapping(BLOCKS_100.as_ptr(), arr_ptr_mut, SIZE)
    }

    initial_blocks
}

const BLOCKS_100: [Block; 100] = [
    1243183713, 182130668, 1454194459, 440815554, 1780603458, 1071104710, 1428186645, 1681358285,
    1862642276, 1921894785, 1630110372, 1819818469, 1517313601, 567804314, 1535738847, 860336423,
    573742082, 1355914565, 137256245, 1480486103, 1726108326, 491128183, 1097611230, 1228313487,
    1388942186, 194712488, 1170756287, 1573897725, 1014265958, 24906164, 2002242887, 513171947,
    105872019, 1519157703, 832221534, 724354983, 716460919, 1416663835, 1507371012, 376054838,
    485083184, 234842817, 859179882, 444965898, 488579921, 837747055, 13964313, 1468067067,
    1657860263, 810492999, 646105966, 1965134910, 511633022, 1497099375, 1447767380, 1684442356,
    687758905, 1060793621, 1863125120, 2087560835, 1893372513, 1287135240, 399718525, 387897017,
    985743452, 1527145208, 677746369, 650102777, 1197688703, 727756928, 1793192148, 1602093392,
    448968042, 1355115532, 852365288, 2130320379, 1177352448, 1515418529, 1802393611, 1708615725,
    237565253, 1510480025, 261223600, 1230659804, 365688338, 357566756, 641730039, 1253172544,
    1263473894, 673016011, 1891853499, 46942072, 1931734276, 128544521, 2034116478, 1575091383,
    1568064634, 1153764404, 1142178529, 1283151306,
];

const BLOCKS_50: [Block; 50] = generate_input_blocks();
const BLOCKS_5: [Block; 5] = generate_input_blocks();

pub fn mine_initialization_bench(c: &mut Criterion) {
    let mut g = c.benchmark_group("Mine::new");
    let id = |n: usize| BenchmarkId::new("Window size", n);

    g.bench_function(id(5), |b| b.iter(|| Mine::new(black_box(BLOCKS_5))));
    g.bench_function(id(50), |b| b.iter(|| Mine::new(black_box(BLOCKS_50))));
    g.bench_function(id(100), |b| b.iter(|| Mine::new(black_box(BLOCKS_100))));
}

pub fn single_block_validation(c: &mut Criterion) {
    let mut g = c.benchmark_group("Mine::try_extend_one");
    let id = |n: usize| BenchmarkId::new("Window size", n);

    let initial_blocks = BLOCKS_100;
    let mut mine = Mine::new(initial_blocks);
    let block_sum = initial_blocks[2] + initial_blocks[4];

    assert_eq!(
        mine.clone().try_extend_one(black_box(block_sum)),
        Ok(()),
        "Sanity check. Making sure the happy path is being benched"
    );

    g.bench_function(id(5), |b| {
        b.iter(|| mine.try_extend_one(black_box(block_sum)))
    });

    let initial_blocks = BLOCKS_100;
    let block_sum = initial_blocks[21] + initial_blocks[38];
    let mut mine = Mine::new(initial_blocks);

    assert_eq!(
        mine.clone().try_extend_one(black_box(block_sum)),
        Ok(()),
        "Sanity check that the happy path is being benched"
    );

    g.bench_function(id(50), |b| {
        b.iter(|| mine.try_extend_one(black_box(block_sum)))
    });

    let initial_blocks = BLOCKS_100;
    let block_sum = initial_blocks[31] + initial_blocks[82];
    let mut mine = Mine::new(initial_blocks);

    assert_eq!(
        mine.clone().try_extend_one(black_box(block_sum)),
        Ok(()),
        "Sanity check that the happy path is being benched"
    );

    g.bench_function(id(100), |b| {
        b.iter(|| mine.try_extend_one(black_box(block_sum)))
    });
}

pub fn many_blocks_validation(c: &mut Criterion) {
    let mut mine = Mine::new(BLOCKS_100);

    let blocks_for_validation: [Block; BLOCKS_100.len() - 1] = BLOCKS_100
        .into_iter()
        .tuple_windows::<(_, _)>()
        .map(|(a, b)| a + b)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    c.bench_function("Many blocks validation. Validation window size: 100", |b| {
        b.iter(|| mine.try_extend(black_box(blocks_for_validation)))
    });
}

criterion_group!(
    benches,
    mine_initialization_bench,
    single_block_validation,
    many_blocks_validation
);
criterion_main!(benches);
