use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use axum::extract::FromRequestParts;
use chdrms_database::permission::{ALL_PERMISSIONS, Permission};
use serde::Deserialize;
use utoipa::ToSchema;
use validify::{ValidationError, ValidationErrors};

use crate::{auth::AuthContext, error::AppError, state::AppState};

pub fn build_permission_map(
    permissions: HashSet<(String, String)>,
) -> HashMap<String, HashSet<String>> {
    let mut map = HashMap::new();
    for (object, action) in permissions {
        map.entry(object)
            .or_insert_with(HashSet::new)
            .insert(action);
    }
    map
}

#[derive(Deserialize, ToSchema, validify::Validate)]
#[validate(validate_permission_ref)]
pub struct PermissionRef {
    pub object: String,
    pub action: String,
}

fn validate_permission_ref(r: &PermissionRef) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();
    for object in ALL_PERMISSIONS {
        if object.object == r.object {
            if object.permissions.contains(&&*r.action) {
                return Ok(());
            } else {
                errors.add(ValidationError::Field {
                    field: Some("action"),
                    code: "invalid_action",
                    params: Box::new(HashMap::new()),
                    message: Some(format!(
                        "`{}` is not a valid action for `{}`",
                        r.action, r.object
                    )),
                    location: "/action".to_string(),
                });
                return Err(errors);
            }
        }
    }
    errors.add(ValidationError::Field {
        field: Some("object"),
        code: "invalid_object",
        params: Box::new(HashMap::new()),
        message: Some(format!("`{}` is not a valid object", r.object)),
        location: "/object".to_string(),
    });
    Err(errors)
}

impl AuthContext {
    pub fn has_permission<P: Permission>(&self) -> bool {
        self.has_permission_raw(P::OBJECT, P::ACTION)
    }
}

pub struct RequirePermission<P: Permission> {
    context: AuthContext,
    _p: PhantomData<P>,
}

impl<P: Permission> AsRef<AuthContext> for RequirePermission<P> {
    fn as_ref(&self) -> &AuthContext {
        &self.context
    }
}

impl<P: Permission> FromRequestParts<AppState> for RequirePermission<P> {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let context = AuthContext::from_request_parts(parts, state).await?;

        if !context.has_permission::<P>() {
            return Err(AppError::Forbidden(format!(
                "missing permission {}->{}",
                P::OBJECT,
                P::ACTION
            )));
        }

        Ok(Self {
            context,
            _p: PhantomData,
        })
    }
}
