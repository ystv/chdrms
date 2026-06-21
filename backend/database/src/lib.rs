use sqlx::PgPool;

pub mod manufacturer;
pub mod permission;
pub mod user;
pub mod user_group;

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

pub enum PatchField<T> {
    Present(T),
    Absent,
}

impl<T> PatchField<T> {
    pub fn is_present(&self) -> bool {
        matches!(self, PatchField::Present(_))
    }

    pub fn into_case_pair(self) -> (bool, Option<T>) {
        (
            self.is_present(),
            match self {
                PatchField::Present(v) => Some(v),
                PatchField::Absent => None,
            },
        )
    }
}

impl<T> PatchField<Option<T>> {
    pub fn into_nullable_case_pair(self) -> (bool, Option<T>) {
        let (is_present, option) = self.into_case_pair();
        (is_present, option.flatten())
    }
}
