use serde::{Deserialize, Deserializer};
use sqlx::PgPool;

pub mod manufacturer;
pub mod permission;
pub mod user;
pub mod user_group;

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PatchField<T> {
    Present(T),
    Absent,
}

impl<T> PatchField<T> {
    pub fn is_present(&self) -> bool {
        matches!(self, PatchField::Present(_))
    }

    pub fn is_absent(&self) -> bool {
        matches!(self, PatchField::Absent)
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

impl<T> Default for PatchField<T> {
    fn default() -> Self {
        PatchField::Absent
    }
}

impl<'de, T> Deserialize<'de> for PatchField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(PatchField::Present(T::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct TestPatch {
        #[serde(default)]
        field: PatchField<String>,
        #[serde(default)]
        nullable_field: PatchField<Option<String>>,
    }

    #[test]
    fn absent_when_missing() {
        let patch: TestPatch = serde_json::from_str("{}").unwrap();
        assert!(patch.field.is_absent());
        assert!(patch.nullable_field.is_absent());
    }

    #[test]
    fn present_some_when_value_given() {
        let patch: TestPatch = serde_json::from_str(r#"{"field": "Hello, World!"}"#).unwrap();
        assert_eq!(
            patch.field,
            PatchField::Present("Hello, World!".to_string())
        );
        assert!(patch.nullable_field.is_absent());
    }

    #[test]
    fn present_none_when_null_given() {
        let patch: TestPatch = serde_json::from_str(r#"{"nullable_field": null}"#).unwrap();
        assert!(patch.field.is_absent());
        assert_eq!(patch.nullable_field, PatchField::Present(None));
    }

    #[test]
    fn non_nullable_rejects_null() {
        let result: Result<TestPatch, _> = serde_json::from_str(r#"{"field": null}"#);
        assert!(result.is_err());
    }
}
