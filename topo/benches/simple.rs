#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion, ParameterizedBenchmark};

fn empty_env(c: &mut Criterion) {
    c.bench_function("call no env", |b| {
        b.iter(|| black_box(topo::root!(topo::Id::current())))
    });
}

fn create_small_env(c: &mut Criterion) {
    c.bench_function("call create small env", |b| {
        b.iter(|| {
            black_box(
                illicit::child_env!(
                    u128 => 10
                )
                .enter(|| topo::root!(topo::Id::current())),
            )
        });
    });
}

fn call_small_env(c: &mut Criterion) {
    c.bench_function("call within small env", |b| {
        illicit::child_env!(u128 => 10).enter(|| {
            topo::call!(b.iter(|| {
                black_box(topo::root!(topo::Id::current()));
            }))
        })
    });
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn topo_bench(b: &mut criterion::Bencher, depth: &usize) {
    macro_rules! mk {
        (go $depth_spec:ident) => {
            illicit::child_env!(u128 => 10).enter(|| {
                topo::root!({
                    mk!(pass $depth_spec 0);
                });
            });
        };
        (pass $depth_spec:ident $call_depth:expr) => {
            topo::root!({
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
            b.iter(|| {
                topo::root!(|| assert_eq!(10, *illicit::Env::get::<u128>().unwrap()))
            });
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

fn from_small_env(c: &mut Criterion) {
    c.bench(
        "topo fns",
        ParameterizedBenchmark::new(
            "call Env::get within small env at depth",
            topo_bench,
            vec![1, 3, 9, 12],
        ),
    );
}

criterion::criterion_group!(
    benches,
    from_small_env,
    create_small_env,
    empty_env,
    call_small_env,
);
criterion::criterion_main!(benches);
