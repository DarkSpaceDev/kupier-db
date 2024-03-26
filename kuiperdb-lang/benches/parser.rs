//--------------------------------------------------------------------------
// (C) Copyright Travis Sharp <travis@darkspace.dev>.  All rights reserved.
//--------------------------------------------------------------------------

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kuiperdb_lang::parser::parse_query;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("query: table identifier", |b| {
        b.iter(|| parse_query(black_box("benchmark_tablename")))
    });

    c.bench_function("query: simple where clause", |b| {
        b.iter(|| parse_query(black_box("benchmark_tablename | where x = y")))
    });

    c.bench_function("query: numeric where", |b| {
        b.iter(|| parse_query(black_box("benchmark_tablename | where 9 = 10")))
    });

    c.bench_function("query: numeric where with negative", |b| {
        b.iter(|| parse_query(black_box("benchmark_tablename | where -9 = 10")))
    });

    c.bench_function("query: complex group (3x groups)", |b| {
        b.iter(|| {
            parse_query(black_box(
                "benchmark_tablename | where x = y and x = y and x = y",
            ))
        })
    });

    c.bench_function("query: complex group (3x groups, 1 or)", |b| {
        b.iter(|| {
            parse_query(black_box(
                "benchmark_tablename | where x = y and (x = y or x = y)",
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
