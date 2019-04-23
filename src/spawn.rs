use futures::{
    future::LocalFutureObj,
    task::{LocalSpawn, SpawnError},
};

// FIXME this is inefficient boooooo

pub trait PrioritySpawn {
    fn spawn_local(&mut self, fut: LocalFutureObj<'static, ()>) -> Result<(), SpawnError>;
    fn child(&self) -> Box<dyn PrioritySpawn>;
}

impl<Exec> PrioritySpawn for Exec
where
    Exec: Clone + LocalSpawn + 'static,
{
    fn spawn_local(&mut self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        LocalSpawn::spawn_local_obj(self, future)
    }

    fn child(&self) -> Box<dyn PrioritySpawn> {
        Box::new(self.clone())
    }
}
