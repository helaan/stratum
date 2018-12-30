use actix::prelude::*;
use actix_web::{error, Error};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};

type DbPool = Pool<ConnectionManager<PgConnection>>;
type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub struct DbExecutor(pub DbPool);

pub fn create_pool(db_url: &str) -> Result<DbPool, PoolError> {
    let mgr = ConnectionManager::<PgConnection>::new(db_url);
    Pool::builder().build(mgr)
}

impl DbExecutor {
    pub fn get_conn(&self) -> Result<DbConnection, Error> {
        self.0.get().map_err(|e| error::ErrorInternalServerError(e))
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

struct Query {
    query: str,
}

impl Message for Query {
    type Result = Result<String, Error>;
}

