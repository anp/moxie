use moxie::{
    cache, once,
    runtime::{Revision, RunLoop},
};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

struct CountDrops {
    num_drops: Arc<AtomicU32>,
}

impl Drop for CountDrops {
    fn drop(&mut self) {
        self.num_drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn issue_238_cache_call_incorrectly_preserves_once_guard() {
    // counts the number of times the once() call below is made
    let once_calls = Arc::new(AtomicU32::new(0));

    // counts the number of times the value returned from once() below is GC'd
    let once_drops = Arc::new(AtomicU32::new(0));

    let (adder, drop_adder) = (once_calls.clone(), once_drops.clone());
    let mut rt = RunLoop::new(move || {
        println!("\n\nrunning loop again");
        let (commit, key) = moxie::state(|| false);

        // caching with the current revision should be a no-op
        println!("calling cache");
        cache(&Revision::current(), |_| {
            println!("entered cache");
            if *commit {
                // this call should be GC'd if commit is false
                println!("calling once");
                once(|| {
                    println!("entered once");
                    adder.fetch_add(1, Ordering::SeqCst);
                    Arc::new(CountDrops { num_drops: drop_adder.clone() })
                });
            }
        });

        key
    });

    // first execution, key is `false` here, no once() call
    let key = rt.run_once();
    assert_eq!(once_calls.load(Ordering::SeqCst), 0);
    assert_eq!(once_drops.load(Ordering::SeqCst), 0);

    // set key to true, once() should execute but not drop
    key.set(true);
    rt.run_once();
    assert_eq!(once_calls.load(Ordering::SeqCst), 1);
    assert_eq!(once_drops.load(Ordering::SeqCst), 0);

    // set key to false, once() should not execute and should get GC'd
    key.set(false);
    rt.run_once();
    assert_eq!(once_calls.load(Ordering::SeqCst), 1);
    assert_eq!(once_drops.load(Ordering::SeqCst), 1);
}
