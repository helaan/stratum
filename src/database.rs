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

impl<I: Send> Handler<Execute<DbConnection, I, Error>> for DbExecutor {
    type Result = Result<I, Error>;
    fn handle(
        &mut self,
        ex: Execute<DbConnection, I, Error>,
        _: &mut Self::Context,
    ) -> Result<I, Error> {
        let conn = self.get_conn()?;
        ex.exec(conn)
    }
}
/* Largely copied from Actix:
 * https://github.com/actix/actix/blob/master/src/msgs.rs
 * Copy needed to add DbExecutor parameter to message
 */
pub struct Execute<P: 'static, I: Send + 'static, E: Send + 'static>(Box<FnExec<P, I, E>>);
impl<P, I: Send, E: Send> Message for Execute<P, I, E> {
    type Result = Result<I, E>;
}

impl<P, I, E> Execute<P, I, E>
where
    I: Send + 'static,
    E: Send + 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(P) -> Result<I, E> + Send + 'static,
    {
        Execute(Box::new(f))
    }

    /// Execute enclosed function
    pub fn exec(self, p: P) -> Result<I, E> {
        self.0.call_box(p)
    }
}

trait FnExec<P, I: Send + 'static, E: Send + 'static>: Send + 'static {
    fn call_box(self: Box<Self>, p: P) -> Result<I, E>;
}

impl<P, I, E, F> FnExec<P, I, E> for F
where
    I: Send + 'static,
    E: Send + 'static,
    F: FnOnce(P) -> Result<I, E> + Send + 'static,
{
    fn call_box(self: Box<Self>, p: P) -> Result<I, E> {
        (*self)(p)
    }
}
