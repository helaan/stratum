use crate::AppState;
use actix_web::middleware::{Middleware, Started};
use actix_web::{error, Error, HttpRequest};
use stratum_db::models::User;

/** CheckRights ensures that there is an user accessing this page and that the
 * user is permitted to access this page.
 *
 * This is done by looking at the rights field in the user:
 * - 1 allows submission
 * - 1000 allows access to the administration panel
 *
 * If users are not allowed, an 404 error is thrown to avoid confirming this
 * resource exists.
 */
pub struct CheckRights {
    pub min_rights: i16,
}

impl Middleware<AppState> for CheckRights {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started, Error> {
        if let Some(user) = req.extensions().get::<User>() {
            if user.rights >= self.min_rights {
                Ok(Started::Done)
            } else {
                Err(error::ErrorNotFound(""))
            }
        } else {
            Err(error::ErrorNotFound(""))
        }
    }
}
