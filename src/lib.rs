use std::sync::Arc;
use axum::{http::{StatusCode, HeaderMap}, extract::{State, self}, Form, response::IntoResponse, Router, routing::{post, get}};
use askama::Template;
use serde::de::DeserializeOwned;

pub type TableHeaderData = &'static str;
pub type TableCellData = String;
pub trait TableDataType: PartialEq {
    fn get_headers() -> Vec<TableHeaderData>;
    fn get_row(&self) -> Vec<TableCellData>;
}

#[derive(serde::Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub value_type: String,
}

#[derive(Template)]
#[template(path="admin-list.html")]
pub struct AdminListTemplate {
    headers: Vec<&'static str>,
    rows: Vec<Vec<String>>,
}

#[derive(Template)]
#[template(path="admin-create.html")]
pub struct AdminCreateTemplate {
    fields: Vec<FieldDef>,
}

pub struct HtmlTemplate<T>(T);

impl<T: Template> IntoResponse for HtmlTemplate<T> {
    fn into_response(self) -> axum::response::Response {
        match self.0.render() {
            Ok(html) => axum::response::Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error rendering template: {}", err)
            ).into_response(),
        }
    }
}

#[async_trait::async_trait]
pub trait RazerModel<AppState: Send + Sync> where Self: serde::Serialize + TableDataType + Clone + Send + Sync {
    async fn list_values(
        state: State<AppState>,
    ) -> Vec<Self>;

    async fn create_value(
        state: State<AppState>,
        input: Self,
    );
}

#[async_trait::async_trait]
pub trait AdminModel: TableDataType where Self: serde::Serialize + TableDataType + Clone + Send + Sync {
    type AppState: Send + Sync;

    fn get_field_definitions() -> Vec<FieldDef>;
    fn model_name() -> String;

pub struct AdminRouter<TState> {
    router: Router<TState>,
}

impl <TState: Send + Sync + Clone + 'static + serde::Serialize> AdminRouter<TState> {
    pub fn new(state: TState) -> Self {
        Self {
            router: Router::new().with_state(state),
        }
    }

    pub fn register<
        TModel: RazerModel<TState> + AdminModel + serde::Serialize + Clone + Send + Sync + DeserializeOwned + 'static>(mut self) -> Self {
        async fn create_api_route<
            TState: Send + Sync + Clone + 'static + serde::Serialize,
            TModel: AdminModel + RazerModel<TState> + Clone + Send + Sync + serde::Serialize + 'static + DeserializeOwned
        > (
            state: State<TState>,
            Form(input): Form<TModel>,
        ) -> (
            StatusCode,
            HeaderMap,
            extract::Json<TModel>
        ) {
            // TODO Remove this as
            <TModel as RazerModel<TState>>::create_value(state, input.clone()).await;

            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", format!("/admin/{}/list", TModel::model_name()).parse().unwrap());
            (
                StatusCode::CREATED,
                headers,
                axum::extract::Json(input)
            )
        }

        async fn create_view_route<TModel: AdminModel>() -> HtmlTemplate<AdminCreateTemplate> {
            let template = AdminCreateTemplate {
                fields: TModel::get_field_definitions(),
            };
            HtmlTemplate(template)
        }

        async fn list_view_route<
            TState: Send + Sync + Clone + 'static + serde::Serialize,
            TModel: AdminModel + RazerModel<TState> + Clone + Send + Sync + serde::Serialize + 'static + DeserializeOwned
        >(
            state: State<TState>,
        ) -> HtmlTemplate<AdminListTemplate> {
            // TODO Remove as
            let values = <TModel as RazerModel<TState>>::list_values(state).await;
            let template = AdminListTemplate {
                headers: TModel::get_headers(),
                rows: values.iter().map(|value| value.get_row()).collect(),
            };
            HtmlTemplate(template)
        }

        let router = Router::<TState>::new()
            .route("/create", get(create_view_route::<TModel>))
            .route("/create", post(create_api_route::<TState, TModel>))
            .route("/list", get(list_view_route::<TState, TModel>));

        self.router = self.router
            .nest(format!("/{}", TModel::model_name()).as_str(), router);

        self
    }
}
