#[macro_use]
extern crate criterion;

use criterion::{Criterion, ParameterizedBenchmark};

fn get_id(c: &mut Criterion) {
    c.bench_function("id from env", |b| {
        topo::call(|| b.iter(|| topo::CallId::current()));
    });
}

fn call_and_get_id(c: &mut Criterion) {
    c.bench_function("call and get id", |b| b.iter(|| topo::call(|| topo::CallId::current())));
}

fn call(c: &mut Criterion) {
    c.bench_function("just a call", |b| b.iter(|| topo::call(|| ())));
}

fn call_topo_fns_to_depth(b: &mut criterion::Bencher, depth: &usize) {
    macro_rules! mk {
        (go $depth_spec:ident) => {
            b.iter(|| topo::call(|| mk!(pass $depth_spec 0)));
        };
        (pass $depth_spec:ident $call_depth:expr) => {
            topo::call(|| {
                mk!(cur $depth_spec ($call_depth + 1));
            });
        };
        (cur twelve $depth:expr) => {
            mk!(pass eleven $depth);
        };
        (cur eleven $depth:expr) => {
            mk!(pass ten $depth);
        };
        (cur ten $depth:expr) => {
            mk!(pass nine $depth);
        };
        (cur nine $depth:expr) => {
            mk!(pass eight $depth);
        };
        (cur eight $depth:expr) => {
            mk!(pass seven $depth);
        };
        (cur seven $depth:expr) => {
            mk!(pass six $depth);
        };
        (cur six $depth:expr) => {
            mk!(pass five $depth);
        };
        (cur five $depth:expr) => {
            mk!(pass four $depth);
        };
        (cur four $depth:expr) => {
            mk!(pass three $depth);
        };
        (cur three $depth:expr) => {
            mk!(pass two $depth);
        };
        (cur two $depth:expr) => {
            mk!(pass one $depth);
        };
        (cur one $depth:expr) => {
            mk!(pass zero ($depth + 1)); // lol what a cheater!
        };
        (cur zero $depth:expr) => {
            assert_eq!($depth - 2, *depth);
        };
    }
    match depth {
        12 => mk!(go twelve),
        11 => mk!(go eleven),
        10 => mk!(go ten),
        9 => mk!(go nine),
        8 => mk!(go eight),
        7 => mk!(go seven),
        6 => mk!(go six),
        5 => mk!(go five),
        4 => mk!(go four),
        3 => mk!(go three),
        2 => mk!(go two),
        1 => mk!(go one),
        _ => unimplemented!(),
    }
}

fn call_depths(c: &mut Criterion) {
    c.bench(
        "call_depths",
        ParameterizedBenchmark::new("topo calls nested to depth", call_topo_fns_to_depth, vec![
            1, 3, 9, 12,
        ]),
    );
}

criterion::criterion_group!(benches, get_id, call_and_get_id, call, call_depths,);
criterion::criterion_main!(benches);
