use crate::article::Article;
use crate::store::model::scan;
use crate::utils::db::establish_connection;
use crate::utils::errors::MyError;

pub fn fetch() -> Result<Vec<Article>, MyError> {
    let pool = establish_connection();
    let conn = pool.get()?;
    scan(&conn)
}
