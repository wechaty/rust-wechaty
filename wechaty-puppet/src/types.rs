use futures::future::{BoxFuture, Future};

pub struct AsyncFnPtr<Payload, Context, Result> {
    func: Box<dyn Fn(Payload, Context) -> BoxFuture<'static, Result> + Send + 'static>,
}

impl<Payload, Context, Result> AsyncFnPtr<Payload, Context, Result>
where
    Payload: 'static,
{
    fn new<Fut, F>(f: F) -> AsyncFnPtr<Payload, Context, Fut::Output>
    where
        F: Fn(Payload, Context) -> Fut + Send + 'static,
        Fut: Future<Output = Result> + Send + 'static,
    {
        AsyncFnPtr {
            func: Box::new(move |t: Payload, ctx: Context| Box::pin(f(t, ctx))),
        }
    }

    pub async fn run(&self, t: Payload, ctx: Context) -> Result {
        (self.func)(t, ctx).await
    }
}

pub trait IntoAsyncFnPtr<Payload, Context, Result>
where
    Payload: 'static,
{
    fn into(self) -> AsyncFnPtr<Payload, Context, Result>;
}

impl<F, Payload, Context, Result, Fut> IntoAsyncFnPtr<Payload, Context, Result> for F
where
    F: Fn(Payload, Context) -> Fut + Send + 'static,
    Payload: 'static,
    Fut: Future<Output = Result> + Send + 'static,
{
    fn into(self) -> AsyncFnPtr<Payload, Context, Result> {
        AsyncFnPtr::new(self)
    }
}
