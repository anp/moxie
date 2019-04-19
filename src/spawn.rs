use futures::{
    future::FutureObj,
    task::{Spawn, SpawnError},
};

pub trait PrioritySpawn {
    fn spawn_obj(&mut self, future: FutureObj<'static, ()>) -> Result<(), SpawnError>;
    fn child(&self) -> Box<dyn PrioritySpawn>;
}

impl<Exec> PrioritySpawn for Exec
where
    Exec: Clone + Spawn + 'static,
{
    fn spawn_obj(&mut self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        Spawn::spawn_obj(self, future)
    }

    fn child(&self) -> Box<dyn PrioritySpawn> {
        Box::new(self.clone())
    }
}
