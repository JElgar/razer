use std::{
    env,
    sync::Arc,
};

use futures::lock::{ Mutex };

use axum::Router;
use diesel::prelude::*;
use diesel::{deserialize::Queryable, Connection, Selectable};
use razer::{AdminRouter, DieselState, RazerModel};
use razer_derive::{AdminInputModel, AdminModel};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::{filter, prelude::__tracing_subscriber_SubscriberExt};
use uuid::Uuid;

mod schema;

#[derive(Clone)]
struct AppState {
    my_classes: Arc<Mutex<Vec<MyClass>>>,
}

impl DieselState for AppState {
    type TConnection = PgConnection;

    fn get_connection(&self) -> Self::TConnection {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, AdminModel)]
struct MyClass {
    id: String,
    title: String,
    description: String,
    other_field: String,
    number: u32,
}

// TODO Use field attributes to set these rather than writing input class separately
#[derive(serde::Deserialize, serde::Serialize, AdminInputModel)]
struct MyClassInput {
    title: String,
    description: String,
    other_field: String,
    number: u32,
}

#[derive(Queryable, Selectable, Identifiable, serde::Serialize, serde::Deserialize, AdminModel)]
#[diesel(table_name = crate::schema::my_models)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MyDieselModel {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Insertable, serde::Deserialize, serde::Serialize, AdminInputModel)]
#[diesel(table_name = crate::schema::my_models)]
pub struct InsertMyDieselModel {
    pub title: String,
    pub body: String,
}

#[async_trait::async_trait]
impl RazerModel<Arc<AppState>, MyClassInput, MyClassInput> for MyClass {
    type IdType = String;

    async fn list_values(
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    ) -> Vec<Self> {
        let lock = state.my_classes.lock().await;
        lock.iter().cloned().collect()
    }

    async fn create_value(
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
        input: MyClassInput,
    ) {
        let data = MyClass {
            id: Uuid::new_v4().to_string(),
            title: input.title,
            description: input.description,
            other_field: input.other_field,
            number: input.number,
        };
        let mut lock = state.my_classes.lock().await;
        lock.push(data.clone());
    }

    async fn get_value(
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
        id: String,
    ) -> Self {
        let lock = state.my_classes.lock().await;
        lock.iter().find(|x| x.id == id).cloned().unwrap()
    }

    async fn update_value(
        axum_state: axum::extract::State<Arc<AppState>>,
        id: String,
        input: MyClassInput,
    ) {
        let value = Self::get_value(axum_state.clone(), id.clone()).await;
        let mut lock = axum_state.my_classes.lock().await;

        lock.iter().find(|x| x.id == id).cloned().unwrap();
        let idx = lock.iter().position(|x| x.id == id).unwrap();

        lock[idx] = MyClass {
            id: value.id,
            title: input.title,
            description: input.description,
            other_field: input.other_field,
            number: input.number,
        };
    }
}

fn diesel_test() {
    use self::schema::my_models::dsl::{ my_models };
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    my_models.find(123)
        .select(MyDieselModel::as_select())
        .first(&mut connection)
        .optional();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = filter::Targets::new()
        .with_target("tower_http::trace::on_response", Level::TRACE)
        .with_target("tower_http::trace::on_request", Level::TRACE)
        .with_default(Level::INFO);

    let subscriber = tracing_subscriber::FmtSubscriber::new().with(filter);
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Initalizing router");

    let app_state = Arc::new(AppState {
        my_classes: Arc::new(Mutex::new(vec![])),
    });

    let admin_router = AdminRouter::new()
        .register::<MyClass, MyClassInput, MyClassInput>()
        .register_diesel_model::<MyDieselModel, InsertMyDieselModel, InsertMyDieselModel>()
        .build();

    let assets_path = std::env::current_dir().unwrap().join("assets");
    let router = Router::new()
        .nest("/admin", admin_router)
        .nest_service("/assets", ServeDir::new(assets_path))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    let port = 7654_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    info!("Listening on port {}", port);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
