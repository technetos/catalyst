use crate::router::RouteHandler;
use futures::future::{self, Future, FutureResult};
use futures_cpupool::CpuPool;
use hyper;
use std::sync::Arc;

/// The server.
pub struct Server<R> {
    /// The router.
    router: Arc<R>,
    /// The pool for threads that handle routes
    pool: CpuPool,
}

impl<R: RouteHandler> Server<R> {
    pub fn new(router: R) -> Server<R> {
        Server {
            router: Arc::new(router),
            // use the number of workers equal to the number of CPUs on the host
            pool: CpuPool::new_num_cpus(),
        }
    }

    pub fn start(&self) {
        /// The address the server is running on.
        let addr = ([127, 0, 0, 1], 8000).into();

        let maker = Worker::new(self.router.clone(), self.pool.clone());

        let server = hyper::Server::bind(&addr)
            .serve(maker)
            .map_err(|e| println!("Server error: {}", e));

        println!("starting server on: {}", addr);

        hyper::rt::run(server);
    }
}

struct Worker<R> {
    router: Arc<R>,
    pool: CpuPool,
}

impl<R: RouteHandler> Worker<R> {
    fn new(router: Arc<R>, pool: CpuPool) -> Worker<R> {
        Worker { router, pool }
    }
}

impl<R: RouteHandler, Ctx> hyper::service::MakeService<Ctx> for Worker<R> {
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = hyper::Error;
    type Service = Worker<R>;
    type Future = FutureResult<Self::Service, Self::Error>;
    type MakeError = hyper::Error;

    fn make_service(&mut self, _: Ctx) -> Self::Future {
        future::ok(Worker::new(self.router.clone(), self.pool.clone()))
    }
}

impl<R: RouteHandler> hyper::service::Service for Worker<R> {
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Future = Box<Future<Item = hyper::Response<Self::ResBody>, Error = Self::Error> + Send>;
    type Error = hyper::Error;

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        Box::new(self.pool.spawn(future::ok(self.router.respond_to(&req))))
    }
}
