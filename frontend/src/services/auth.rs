use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: ReadSignal<Option<AuthUser>>,
    pub is_authenticated: ReadSignal<bool>,
    pub login: WriteSignal<Option<AuthUser>>,
    pub logout: WriteSignal<()>,
}

pub fn provide_auth_context() -> AuthContext {
    let (user, set_user) = create_signal(None::<AuthUser>);
    let (login_trigger, set_login_trigger) = create_signal(None::<AuthUser>);
    let (logout_trigger, set_logout_trigger) = create_signal(());

    // Check for existing session on load
    create_effect(move |_| {
        if let Ok(stored) = LocalStorage::get::<AuthUser>("auth_user") {
            set_user.set(Some(stored));
        }
    });

    // Handle login
    create_effect(move |_| {
        if let Some(auth_user) = login_trigger.get() {
            LocalStorage::set("auth_user", &auth_user).unwrap_or_default();
            set_user.set(Some(auth_user));
        }
    });

    // Handle logout
    create_effect(move |_| {
        if logout_trigger.get() != () {
            LocalStorage::delete("auth_user");
            set_user.set(None);
        }
    });

    let is_authenticated = Signal::derive(move || user.get().is_some());

    AuthContext {
        user,
        is_authenticated,
        login: set_login_trigger,
        logout: set_logout_trigger,
    }
}

pub fn use_auth_context() -> AuthContext {
    use_context::<AuthContext>().expect("Auth context not provided")
}
