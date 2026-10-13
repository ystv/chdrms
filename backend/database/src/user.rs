use std::collections::HashSet;

use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{define_id, permission::define_permissions};

#[derive(Debug, FromRow)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
    pub is_admin: bool,
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
            r#"INSERT INTO users(email, name)
            VALUES ($1, $2)
            RETURNING id AS "id: _", email, name, is_admin;"#,
            create.email,
            create.name
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_by_id(
        txn: &mut sqlx::PgTransaction<'_>,
        id: UserId,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            User,
            r#"SELECT id AS "id: _", email, name, is_admin
            FROM users
            WHERE id = $1;"#,
            id as _,
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn get_by_session(
        txn: &mut sqlx::PgTransaction<'_>,
        token: Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            User,
            r#"SELECT
                users.id AS "id: _", users.email, users.name, users.is_admin
            FROM user_sessions
            JOIN users ON user_sessions.user_id = users.id
            WHERE user_sessions.token = $1;"#,
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
            r#"SELECT
                users.id AS "id: _", users.email, users.name, users.is_admin
            FROM user_identities
            JOIN users ON user_identities.user_id = users.id
            WHERE
                user_identities.provider = $1
                AND user_identities.provider_id = $2;"#,
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
            r#"SELECT
                id AS "id: _", email, name, is_admin
            FROM users
            WHERE
                email = $1;"#,
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
            self.id as _,
            provider,
            provider_id
        )
        .execute(&mut **txn)
        .await?;
        Ok(())
    }

    pub async fn list_permissions(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
    ) -> sqlx::Result<HashSet<(String, String)>> {
        let permissions = sqlx::query!(
            "
        SELECT
            DISTINCT ON (group_permissions.object, group_permissions.action)
            group_permissions.object, group_permissions.action
        FROM user_groups
        JOIN group_permissions
            ON group_permissions.group_id = user_groups.group_id
        WHERE user_groups.user_id = $1;
        ",
            self.id as _
        )
        .fetch_all(&mut **txn)
        .await?;
        Ok(permissions
            .into_iter()
            .map(|r| (r.object, r.action))
            .collect())
    }

    /// Create a new session for this user, returning the token
    pub async fn create_session(&self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Uuid> {
        let result = sqlx::query!(
            "INSERT INTO user_sessions(user_id) VALUES ($1) RETURNING token;",
            self.id as _
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

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<User>> {
        sqlx::query_as!(
            User,
            r#"SELECT id AS "id: _", email, name, is_admin FROM users;"#
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn count(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<i64> {
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!"
            FROM users;"#
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn set_admin(
        self,
        admin: bool,
        txn: &mut sqlx::PgTransaction<'_>,
    ) -> sqlx::Result<User> {
        sqlx::query_as!(
            Self,
            r#"UPDATE users
            SET is_admin = $2
            WHERE id = $1
            RETURNING id AS "id: _", email, name, is_admin;"#,
            self.id as _,
            admin,
        )
        .fetch_one(&mut **txn)
        .await
    }
}

define_permissions!("users" => List, Invite, Manage);
define_id!(UserId);
