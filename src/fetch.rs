use crate::article::Article;
use crate::store::model::scan;
use crate::utils::db::establish_connection;
use crate::utils::errors::MyError;
use diesel::MysqlConnection;

pub fn fetch(conn: &MysqlConnection) -> Result<Vec<Article>, MyError> {
    scan(&conn)
}
