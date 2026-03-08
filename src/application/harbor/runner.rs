use futures::future::BoxFuture;

pub trait HarborJobRunner: Send + Sync {
    fn spawn(&self, task: BoxFuture<'static, ()>);
}

#[derive(Default)]
pub struct TokioHarborJobRunner;

impl HarborJobRunner for TokioHarborJobRunner {
    fn spawn(&self, task: BoxFuture<'static, ()>) {
        tokio::spawn(task);
    }
}
