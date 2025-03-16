use razer_core::*;
use razer_derive::AdminPanel;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use axum::Router;
use tokio::net::TcpListener;
use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

// Simple in-memory database for the example
static USERS: Lazy<Mutex<HashMap<i32, User>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(1, User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        password: "secret".to_string(),
        created_at: Utc::now(),
    });
    Mutex::new(m)
});

#[derive(Debug, Clone, Serialize, Deserialize, AdminPanel)]
#[admin(
    name = "Users",
    path = "users",
    list_fields = ["id", "name", "email", "created_at"],
    create_fields = ["name", "email", "password"],
    edit_fields = ["name", "email"],
    search_fields = ["name", "email"],
    filter_fields = ["created_at"]
)]
struct User {
    id: i32,
    name: String,
    email: String,
    #[admin(sensitive)]
    password: String,
    created_at: DateTime<Utc>,
}

impl User {
    async fn find_by_id(id: i32) -> Result<Self, AdminError> {
        USERS.lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| AdminError::NotFound("User not found".to_string()))
    }

    async fn find_all(_params: ListParams) -> Result<Vec<Self>, AdminError> {
        let users = USERS.lock()
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        println!("Debug - find_all called, returning users: {:?}", users);
        Ok(users)
    }

    async fn create(mut data: Self) -> Result<Self, AdminError> {
        let mut users = USERS.lock().unwrap();
        let id = users.len() as i32 + 1;
        data.id = id;
        data.created_at = Utc::now();
        users.insert(id, data.clone());
        Ok(data)
    }

    async fn update(id: i32, data: Self) -> Result<Self, AdminError> {
        let mut users = USERS.lock().unwrap();
        if let Some(user) = users.get_mut(&id) {
            user.name = data.name;
            user.email = data.email;
            Ok(user.clone())
        } else {
            Err(AdminError::NotFound("User not found".to_string()))
        }
    }

    async fn delete(id: i32) -> Result<(), AdminError> {
        let mut users = USERS.lock().unwrap();
        if users.remove(&id).is_some() {
            Ok(())
        } else {
            Err(AdminError::NotFound("User not found".to_string()))
        }
    }
}

#[tokio::main]
async fn main() {
    // Create the admin panel
    let admin = Admin::new()
        .register::<User>()
        .with_title("My Application Admin")
        .with_theme(Theme::default());

    // Create Axum router with admin routes
    let app = Router::new()
        .nest("/admin", admin.router());

    println!("Admin panel running at http://localhost:3000/admin");

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
} 