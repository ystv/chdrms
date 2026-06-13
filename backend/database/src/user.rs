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
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        create: UserCreation,
    ) -> sqlx::Result<Self> {
        // TODO: validate email?
        sqlx::query_as!(
            User,
            "INSERT INTO users(email, name) VALUES ($1, $2) RETURNING id, email, name;",
            create.email,
            create.name
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_by_id(
        txn: &mut sqlx::PgTransaction<'_>,
        id: Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(User, "SELECT id, email, name FROM users WHERE id = $1", id)
            .fetch_optional(&mut **txn)
            .await
    }

    pub async fn get_by_session(
        txn: &mut sqlx::PgTransaction<'_>,
        token: Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            User,
            r#"
            SELECT
                users.id, users.email, users.name
            FROM user_sessions
            JOIN users ON user_sessions.user_id = users.id
            WHERE user_sessions.token = $1;
            "#,
            token
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn get_by_external_id(
        txn: &mut sqlx::PgTransaction<'_>,
        provider: &str,
        provider_id: &str,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            User,
            r#"
            SELECT
                users.id, users.email, users.name
            FROM user_identities
            JOIN users ON user_identities.user_id = users.id
            WHERE
                user_identities.provider = $1
                AND user_identities.provider_id = $2;
            "#,
            provider,
            provider_id,
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn get_by_email(
        txn: &mut sqlx::PgTransaction<'_>,
        email: &str,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            User,
            r#"
            SELECT
                id, email, name
            FROM users
            WHERE
                email = $1
            "#,
            email,
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn attach_external_id(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        provider: &str,
        provider_id: &str,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO user_identities(user_id, provider, provider_id) VALUES ($1, $2, $3);",
            self.id,
            provider,
            provider_id
        )
        .execute(&mut **txn)
        .await?;
        Ok(())
    }

    /// Create a new session for this user, returning the token
    pub async fn create_session(&self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Uuid> {
        let result = sqlx::query!(
            "INSERT INTO user_sessions(user_id) VALUES ($1) RETURNING token;",
            &self.id
        )
        .fetch_one(&mut **txn)
        .await?;
        Ok(result.token)
    }

    pub async fn destroy_session(
        txn: &mut sqlx::PgTransaction<'_>,
        token: Uuid,
    ) -> sqlx::Result<bool> {
        let res = sqlx::query!("DELETE FROM user_sessions WHERE token = $1;", token)
            .execute(&mut **txn)
            .await?;
        Ok(res.rows_affected() != 0)
    }
}
