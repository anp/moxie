#[macro_use]
extern crate criterion;

use criterion::{Criterion, ParameterizedBenchmark};
use moxie::embed::{Revision, Runtime};

criterion::criterion_group!(runtime, run_once_empty, run_repeated);
criterion::criterion_main!(runtime);

fn run_once_empty(c: &mut Criterion) {
    let mut rt = Runtime::new(|| Revision::current());
    c.bench_function("run_once_empty", |b| b.iter(|| rt.run_once()));
}

fn run_n_times_empty(b: &mut criterion::Bencher, n: &usize) {
    let mut rt = Runtime::new(|| Revision::current());
    b.iter(|| {
        for _ in 0..*n {
            rt.run_once();
        }
    });
}

fn run_repeated(c: &mut Criterion) {
    c.bench(
        "run_repeated",
        ParameterizedBenchmark::new(
            "run_once called several times",
            run_n_times_empty,
            vec![2, 7, 23],
        ),
    );
}
