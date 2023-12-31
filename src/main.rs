// TODO Pick an orm and create a type
//
// TODO Pick an API framework and create an admin endpoint
//
// TODO Somehow generate admin page for the type
// https://github.com/mitsuhiko/minijinja
//
//
// https://github.com/silkenweb/silkenweb/blob/main/examples/htmx-axum/index.html

use std::{sync::{Mutex, Arc}, env};

use axum::Router;
use diesel::{prelude::*, associations::HasTable};
use diesel::{Connection, Selectable, deserialize::Queryable};
use dotenvy::dotenv;
use razer_derive::{AdminModel, AdminInputModel};
use tower_http::services::ServeDir;
use tracing::{info, Level};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, filter};
// TODO Don't have the same name for the trait and macro!
use razer::{RazerModel, AdminRouter, DieselState};
use uuid::Uuid;

mod schema;

#[derive(Clone)]
struct AppState {
    my_classes: Arc<Mutex<Vec<MyClass>>>,
}

impl DieselState for AppState {
    type TConnection = PgConnection;

    fn get_connection(&self) -> Self::TConnection {
        PgConnection::establish(&dotenvy::var("DATABASE_URL").unwrap()).unwrap()
    }
}

// type RealAppState = Arc<AppState>;

// impl DieselState for RealAppState {
//     type TConnection = PgConnection;
// 
//     fn get_connection(&self) -> Self::TConnection {
//         PgConnection::establish(&dotenvy::var("DATABASE_URL").unwrap()).unwrap()
//     }
// }

// Create a macro which iterates over the fields of the struct
// based on the field type it can setup the form... Does that make sense maybe the form can do that
// automatically if I just idk...?
// #[derive(serde::Deserialize)]
// TODO Split AdminModel -> InputModel and Model (table stuff)
#[derive(PartialEq, Clone, serde::Deserialize, serde::Serialize, razer_derive::AdminModel)]
struct MyClass {
    id: String,
    title: String,
    description: String,
    other_field: String,
    number: u32,
}

#[derive(PartialEq, Clone, serde::Deserialize, serde::Serialize, razer_derive::AdminInputModel)]
struct MyClassInput {
    title: String,
    description: String,
    other_field: String,
    number: u32,
}

#[derive(Queryable, Selectable, Identifiable, serde::Serialize, PartialEq, Clone, AdminModel)]
#[diesel(table_name = crate::schema::my_models)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MyDieselModel {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Insertable, serde::Deserialize, serde::Serialize, Clone, AdminInputModel)]
#[diesel(table_name = crate::schema::my_models)]
pub struct InsertMyDieselModel {
    pub title: String,
    pub body: String,
}

#[async_trait::async_trait]
impl RazerModel<Arc<AppState>, MyClassInput> for MyClass {
    async fn list_values(
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    ) -> Vec<Self> {
        let lock = state.my_classes.lock().unwrap();
        lock.iter().cloned().collect()
    }

    async fn create_value(
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
        input: MyClassInput,
    ) -> Self {
        let data = MyClass {
            id: Uuid::new_v4().to_string(),
            title: input.title,
            description: input.description,
            other_field: input.other_field,
            number: input.number,
        };
        let mut lock = state.my_classes.lock().unwrap();
        lock.push(data.clone());
        data
    }
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn test_query() {
    use self::schema::my_models::dsl::{my_models, published};

    let connection = &mut establish_connection();
    let results = my_models
        // .filter(published.eq(true))
        .limit(5)
        .select(MyDieselModel::as_select())
        .load(connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.title);
        println!("-----------\n");
        println!("{}", post.body);
    }
}

pub fn create_my_model(conn: &mut PgConnection, title: &str, body: &str) -> MyDieselModel {
    use crate::schema::my_models;

    let new_post = InsertMyDieselModel { title: title.to_string(), body: body.to_string() };

    diesel::insert_into(my_models::table)
        .values(&new_post)
        .returning(MyDieselModel::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
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
        my_classes: Arc::new(Mutex::new(vec![])),
    });

    let admin_router = AdminRouter::new()
        .register::<MyClass, MyClassInput>()
        .register_diesel_model::<MyDieselModel, InsertMyDieselModel>()
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
