// TODO Pick an orm and create a type
//
// TODO Pick an API framework and create an admin endpoint
//
// TODO Somehow generate admin page for the type
// https://github.com/mitsuhiko/minijinja
//
//
// https://github.com/silkenweb/silkenweb/blob/main/examples/htmx-axum/index.html

use std::sync::{Mutex, Arc};

use axum::Router;
use tower_http::services::ServeDir;
use tracing::{info, Level};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, filter};
// TODO Don't have the same name for the trait and macro!
use razer::{RazerModel, AdminRouter};

struct AppState {
    my_classes: Mutex<Vec<MyClass>>,
}

// Create a macro which iterates over the fields of the struct
// based on the field type it can setup the form... Does that make sense maybe the form can do that
// automatically if I just idk...?
// #[derive(serde::Deserialize)]
#[derive(PartialEq, Clone, serde::Deserialize, serde::Serialize, razer_derive::AdminModel)]
struct MyClass {
    title: String,
    description: String,
    other_field: String,
    number: u32,
}

// TODO
struct MyDieselModel {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[async_trait::async_trait]
impl RazerModel<Arc<AppState>> for MyClass {
    async fn list_values(
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    ) -> Vec<Self> {
        let lock = state.my_classes.lock().unwrap();
        lock.iter().cloned().collect()
    }

    async fn create_value(
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
        input: Self,
    ) {
        let mut lock = state.my_classes.lock().unwrap();
        lock.push(input.clone());
    }
}

// TODO When referencing this trait in the dervice, use the full path (e.g. razer::admin::AdminModel)
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
        my_classes: Mutex::new(vec![]),
    });

    let admin_router = AdminRouter::new()
        .register::<MyClass>()
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
