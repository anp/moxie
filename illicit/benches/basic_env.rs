#[macro_use]
extern crate criterion;

use criterion::Criterion;

fn enter_small_env(c: &mut Criterion) {
    c.bench_function("enter a small illicit env", |b| {
        b.iter(|| illicit::Layer::new().offer(10u128).enter(|| ()));
    });
}

criterion::criterion_group!(benches, enter_small_env,);
criterion::criterion_main!(benches);
