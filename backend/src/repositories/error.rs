#[derive(Debug)]
pub enum RepositoryError {
    Database(sqlx::Error),
}
