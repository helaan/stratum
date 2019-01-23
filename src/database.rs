use actix::prelude::*;
use actix_web::{error, Error};
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use diesel::PgConnection;
use std::marker::Send;

type DbPool = Pool<ConnectionManager<PgConnection>>;
type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub struct DbExecutor(pub DbPool);

pub fn create_pool(db_url: &str) -> Result<DbPool, PoolError> {
    let mgr = ConnectionManager::<PgConnection>::new(db_url);
    Pool::builder().build(mgr)
}

impl DbExecutor {
    pub fn get_conn(&self) -> Result<DbConnection, Error> {
        self.0.get().map_err(error::ErrorInternalServerError)
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

/* Largely copied from Actix:
 * https://github.com/actix/actix/blob/master/src/msgs.rs
 * Copy needed to add DbExecutor parameter to message
 */
pub struct Execute<I: Send + 'static = (), E: Send + 'static = ()>(Box<FnExec<I, E>>);
impl<I: Send, E: Send> Message for Execute<I, E> {
    type Result = Result<I, E>;
}

impl<I, E> Execute<I, E>
where
    I: Send + 'static,
    E: Send + 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&&mut DbExecutor) -> Result<I, E> + Send + 'static,
    {
        Execute(Box::new(f))
    }

    /// Execute enclosed function
    pub fn exec(self, s: &&mut DbExecutor) -> Result<I, E> {
        self.0.call_box(s)
    }
}

trait FnExec<I: Send + 'static, E: Send + 'static>: Send + 'static {
    fn call_box(self: Box<Self>, s: &&mut DbExecutor) -> Result<I, E>;
}

impl<I, E, F> FnExec<I, E> for F
where
    I: Send + 'static,
    E: Send + 'static,
    F: FnOnce(&&mut DbExecutor) -> Result<I, E> + Send + 'static,
{
    fn call_box(self: Box<Self>, s: &&mut DbExecutor) -> Result<I, E> {
        (*self)(s)
    }
}

impl<I: Send, E: Send> Handler<Execute<I, E>> for DbExecutor {
    type Result = Result<I, E>;
    fn handle(&mut self, ex: Execute<I, E>, _: &mut Self::Context) -> Result<I, E> {
        ex.exec(&self)
    }
}
