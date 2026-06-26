use std::collections::HashSet;

use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{define_id, permission::define_permissions, user::User};

#[derive(Debug, FromRow)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
}

impl Group {
    pub async fn create(txn: &mut sqlx::PgTransaction<'_>, name: &str) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Group,
            r#"INSERT INTO groups(name) VALUES ($1) RETURNING id AS "id: _", name;"#,
            name
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(Group, r#"SELECT id AS "id: _", name FROM groups;"#)
            .fetch_all(&mut **txn)
            .await
    }

    pub async fn get_by_id(
        txn: &mut sqlx::PgTransaction<'_>,
        id: Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Group,
            r#"SELECT id AS "id: _", name FROM groups WHERE id = $1;"#,
            id
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn list_members(&self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<User>> {
        sqlx::query_as!(
            User,
            r#"SELECT
                users.id AS "id: _", users.email, users.name, users.is_admin
            FROM user_groups
            JOIN users ON user_groups.user_id = users.id
            WHERE
                user_groups.group_id = $1;"#,
            self.id as _
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn add_member(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        user: User,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            "INSERT INTO user_groups(group_id, user_id)
            VALUES($1, $2)
            ON CONFLICT DO NOTHING;",
            self.id as _,
            user.id as _
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn remove_member(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        user: User,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM user_groups
            WHERE group_id = $1 AND user_id = $2;",
            self.id as _,
            user.id as _
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn list_permissions(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
    ) -> sqlx::Result<HashSet<(String, String)>> {
        let permissions = sqlx::query!(
            "SELECT group_permissions.object, group_permissions.action
            FROM group_permissions
            WHERE group_permissions.group_id = $1;",
            self.id.0
        )
        .fetch_all(&mut **txn)
        .await?;
        Ok(permissions
            .into_iter()
            .map(|r| (r.object, r.action))
            .collect())
    }

    pub async fn add_permission(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        object: &str,
        action: &str,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            "INSERT INTO group_permissions(group_id, object, action)
            VALUES($1, $2, $3)
            ON CONFLICT DO NOTHING;",
            self.id as _,
            object,
            action,
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn remove_permission(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        object: &str,
        action: &str,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM group_permissions
            WHERE group_id = $1 AND object = $2 AND action = $3;",
            self.id as _,
            object,
            action,
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() == 1)
    }
}

define_permissions!("user_group" => List, Manage, ManageMembers, ManagePermissions);
define_id!(GroupId);
