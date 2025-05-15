#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use moxie::{
    once,
    runtime::{Revision, RunLoop},
};
use std::rc::Rc;

criterion::criterion_group!(runtime, once_from_store, run_empty, run_repeated);
criterion::criterion_main!(runtime);

fn once_from_store(c: &mut Criterion) {
    let mut rt = RunLoop::new(|| once(|| Rc::new(vec![0; 1_000_000])));
    rt.run_once();
    c.bench_function("1mb vec cached", |b| b.iter(|| rt.run_once()));
}

fn run_empty(c: &mut Criterion) {
    let mut rt = RunLoop::new(Revision::current);
    c.bench_function("run_empty", |b| b.iter(|| rt.run_once()));
}

fn run_n_times_empty(b: &mut criterion::Bencher, n: &usize) {
    let mut rt = RunLoop::new(Revision::current);
    b.iter(|| {
        for _ in 0..*n {
            rt.run_once();
        }
    });
}

fn run_repeated(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_repeated");
    for input in &[2, 7, 23] {
        group.bench_with_input(BenchmarkId::from_parameter(input), input, run_n_times_empty);
    }
    group.finish();
}
