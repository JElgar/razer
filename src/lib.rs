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
#[template(path="admin.html")]
pub struct AdminTemplate {
    models: Vec<ModelInfo>
}

#[derive(Template)]
#[template(path="admin-list.html")]
pub struct AdminListTemplate {
    headers: Vec<&'static str>,
    rows: Vec<Vec<String>>,
    create_view_route: String,
}

#[derive(Template)]
#[template(path="admin-create.html")]
pub struct AdminCreateTemplate {
    fields: Vec<FieldDef>,
    create_endpoint: String,
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
pub trait AdminModel where Self: serde::Serialize + TableDataType + Clone + Send + Sync {
    fn get_field_definitions() -> Vec<FieldDef>;
    fn model_name() -> String;

    fn get_list_url() -> String {
        format!("/admin/{}/list", Self::model_name())
    }

    fn get_create_url() -> String {
        format!("/admin/{}/create", Self::model_name())
    }
}

#[derive(Clone)]
struct ModelInfo {
    pub name: String,
    pub list_view_route: String,
}

pub struct AdminRouter<TState> {
    router: Router<TState>,
    models: Vec<ModelInfo>,
}

impl <TState: Send + Sync + Clone + 'static> AdminRouter<TState> {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            models: Vec::new(),
        }
    }

    pub fn register<
        TModel: RazerModel<TState> + AdminModel + serde::Serialize + Clone + Send + Sync + DeserializeOwned + 'static>(mut self) -> Self {
        async fn create_api_route<
            TState: Send + Sync + Clone + 'static,
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
            headers.insert("HX-Redirect", TModel::get_list_url().parse().unwrap());
            (
                StatusCode::CREATED,
                headers,
                axum::extract::Json(input)
            )
        }

        async fn create_view_route<TModel: AdminModel>() -> HtmlTemplate<AdminCreateTemplate> {
            let template = AdminCreateTemplate {
                fields: TModel::get_field_definitions(),
                create_endpoint: TModel::get_create_url(),
            };
            HtmlTemplate(template)
        }

        async fn list_view_route<
            TState: Send + Sync + Clone + 'static,
            TModel: AdminModel + RazerModel<TState> + Clone + Send + Sync + serde::Serialize + 'static + DeserializeOwned
        >(
            state: State<TState>,
        ) -> HtmlTemplate<AdminListTemplate> {
            // TODO Remove as
            let values = <TModel as RazerModel<TState>>::list_values(state).await;
            let template = AdminListTemplate {
                headers: TModel::get_headers(),
                rows: values.iter().map(|value| value.get_row()).collect(),
                create_view_route: TModel::get_create_url(),
            };
            HtmlTemplate(template)
        }

        let router = Router::<TState>::new()
            .route("/create", get(create_view_route::<TModel>))
            .route("/create", post(create_api_route::<TState, TModel>))
            .route("/list", get(list_view_route::<TState, TModel>));

        self.router = self.router
            .nest(format!("/{}", TModel::model_name()).as_str(), router);

        self.models.push(ModelInfo {
            name: TModel::model_name(),
            list_view_route: TModel::get_list_url(),
        });

        self
    }

    pub fn build(self) -> Router<TState> {
        self.router.route("/", get(|| async {
            // TODO Remove as
            let template = AdminTemplate {
                models: self.models,
            };
            HtmlTemplate(template)
        }))
    }
}
