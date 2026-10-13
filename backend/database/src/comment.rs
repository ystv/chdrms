use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[schema]
struct Comment {
    #[schema(generated, immutable)]
    id: Uuid,
    #[schema(generated)]
    archived_at: Option<DateTime<Utc>>,

    title: String,
    content: String,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: Uuid,
}

impl Comment {
    pub(super) async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        comment: CreateComment,
    ) -> sqlx::Result<Comment> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO comments(title, content, created_by)
            VALUES ($1, $2, $3)
            RETURNING id, archived_at, title, content, created_at, created_by;"#,
            comment.title,
            comment.content,
            comment.created_by,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn update(
        self,
        txn: &mut sqlx::PgTransaction<'_>,
        comment: UpdateComment,
    ) -> sqlx::Result<Comment> {
        sqlx::query_as!(
            Self,
            r#"UPDATE comments
            SET archived_at = $2, title = $3, content = $4
            WHERE id = $1
            RETURNING id, archived_at, title, content, created_at, created_by;"#,
            self.id,
            comment.archived_at,
            comment.title,
            comment.content,
        )
        .fetch_one(&mut **txn)
        .await
    }
}
