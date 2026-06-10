use uuid::Uuid;

pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

pub struct UserCreation {
    pub email: String,
    pub name: String,
}

impl User {
    pub async fn create(txn: &mut sqlx::PgTransaction<'_>, create: UserCreation) -> sqlx::Result<Self> {
        // TODO: validate email?
        sqlx::query_as!(User, "INSERT INTO users(email, name) VALUES ($1, $2) RETURNING id, email, name;", create.email, create.name)
            .fetch_one(&mut **txn)
            .await
    }

    pub async fn get_by_id(txn: &mut sqlx::PgTransaction<'_>, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(User, "SELECT id, email, name FROM users WHERE id = $1", id)
            .fetch_optional(&mut **txn)
            .await
    }

    pub async fn get_by_session(txn: &mut sqlx::PgTransaction<'_>, token: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(User, "SELECT users.id, users.email, users.name FROM user_sessions JOIN users ON user_sessions.user_id = users.id WHERE user_sessions.token = $1;", token)
            .fetch_optional(&mut **txn)
            .await
    }

    /// Create a new session for this user, returning the token
    pub async fn create_session(&self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Uuid> {
        let result = sqlx::query!("INSERT INTO user_sessions(user_id) VALUES ($1) RETURNING token;", &self.id)
            .fetch_one(&mut **txn)
            .await?;
        Ok(result.token)
    }
}
