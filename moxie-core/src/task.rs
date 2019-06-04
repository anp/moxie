use {
    super::*,
    futures::task::{LocalSpawn, SpawnError},
    std::{
        future::Future,
        panic::{catch_unwind, AssertUnwindSafe},
        task::Waker,
    },
};

flow_local!(static WAKER: Waker = null_waker());
flow_local!(static SPAWNER: Box<dyn Spawn + Send> = null_spawner());

pub async fn runloop<C: Component + Clone + 'static>(
    root: C,
    spawner: impl Spawn + Send + 'static,
) {
    // make sure we can be woken back up and exited
    std::future::get_task_context(|cx| WAKER.set(cx.waker().clone()));
    SPAWNER.set(Box::new(spawner));

    loop {
        let root = AssertUnwindSafe(root.clone());
        if let Err(e) = catch_unwind(move || {
            let root = root.clone();
            trace!("composing");
            show!(root);
        }) {
            error!("error composing: {:?}", e);
            // TODO soft restart (reset state, recordings, etc)
        }
        futures::pending!();
    }
}

pub trait Spawn: 'static {
    fn spawn_local(
        &mut self,
        fut: Box<dyn Future<Output = ()> + 'static>,
    ) -> Result<(), SpawnError>;
    fn child(&self) -> Box<dyn Spawn>;
}

impl<Exec> Spawn for Exec
where
    Exec: 'static + Clone + LocalSpawn,
{
    fn spawn_local(
        &mut self,
        fut: Box<dyn Future<Output = ()> + 'static>,
    ) -> Result<(), SpawnError> {
        LocalSpawn::spawn_local_obj(self, fut.into())
    }

    fn child(&self) -> Box<dyn Spawn> {
        Box::new(self.clone())
    }
}

fn null_waker() -> Waker {
    unimplemented!()
}

fn null_spawner() -> Box<dyn Spawn + Send> {
    unimplemented!()
}
