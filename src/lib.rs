use std::{future::Future, sync::Arc};

use askama::Template;
use axum::{
    extract::{State, Path},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use diesel::{
    associations::HasTable,
    helper_types::{AsSelect, Limit, Select},
    query_builder::{InsertStatement, QueryId},
    query_dsl::methods::{ExecuteDsl, LimitDsl, LoadQuery, SelectDsl, FindDsl}, Connection, Insertable, SelectableHelper, Identifiable, RunQueryDsl, connection::LoadConnection, OptionalExtension,
};
use serde::de::DeserializeOwned;
use std::ops::Deref;

#[derive(Clone, serde::Deserialize)]
pub struct FieldConfig<T> {
    pub attribute_name: String,
    pub display_name: String,

    // TODO
    pub default_value: Option<T>,
    // pub display_description: String,

    // Maybe not?
    // pub read_only: bool,
}

#[derive(Clone, serde::Deserialize)]
pub struct FieldWithValue<T> {
    pub field_config: FieldConfig<T>,
    pub value: T,
}

#[derive(Clone, serde::Deserialize)]
pub enum FieldDef {
    Text(FieldConfig<String>),
    Number(FieldConfig<i64>),
    Boolean(FieldConfig<bool>),
}

#[derive(Clone, serde::Deserialize)]
pub enum FieldValue {
    Text(FieldWithValue<String>),
    Number(FieldWithValue<i64>),
    Boolean(FieldWithValue<bool>),
}

impl FieldDef {
    pub fn get_field_display_name(&self) -> String {
        match self {
            FieldDef::Text(config) => config.display_name.clone(),
            FieldDef::Number(config) => config.display_name.clone(),
            FieldDef::Boolean(config) => config.display_name.clone(),
        }
    }
}

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate {
    models: Vec<ModelViewInfo>,
}

#[derive(Template)]
#[template(path = "admin-list.html")]
pub struct AdminListTemplate {
    // headers: Vec<String>,
    // colum_types: Vec<String>,

    columns: Vec<FieldDef>,
    rows: Vec<Vec<String>>,
    create_view_route: String,
    models: Vec<ModelViewInfo>,
}

#[derive(Template)]
#[template(path = "admin-view.html")]
pub struct AdminViewTemplate {
    models: Vec<ModelViewInfo>,
    fields: Vec<FieldValue>,
}

#[derive(Template)]
#[template(path = "admin-create.html")]
pub struct AdminCreateTemplate {
    models: Vec<ModelViewInfo>,
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
                format!("Error rendering template: {}", err),
            )
                .into_response(),
        }
    }
}

pub trait AdminModelBase {
    fn get_field_definitions() -> Vec<FieldDef>;
}

pub trait AdminInputModel: serde::Serialize + DeserializeOwned + Send + Sync + 'static + AdminModelBase {
}

pub trait AdminModel: serde::Serialize + AdminModelBase {
    fn model_name() -> &'static str;
    fn get_row(&self) -> Vec<String>;
    fn get_field_values(&self) -> Vec<FieldValue>;
}

// TODO Look for existing methods to convert to javascript type
// TODO Need way to differentiate between short and long text
// TODO Need way to do validation on number types
enum FieldType {
    Number,
    Boolean,
    String,
}

#[async_trait::async_trait]
pub trait RazerModel<AppState: Clone + Send + Sync + 'static, InsertionModel: AdminInputModel>:
    AdminModel
{
    type IdType: 'static + DeserializeOwned + Send;

    async fn list_values(state: State<AppState>) -> Vec<Self>
    where
        Self: Sized;

    async fn create_value(state: State<AppState>, input: InsertionModel);

    // TODO Does id make sense? Will this always be the identifier?
    async fn get_value(state: State<AppState>, id: Self::IdType) -> Self;

    // TODO Can these methods be separated out so its optional which are implemeneted?
    // TODO
    // async fn update_value(state: State<AppState>, input: UpdateModel);
}

#[derive(Clone, Debug)]
struct ModelViewInfo {
    pub name: &'static str,
    pub list_view_route: String,
}

struct ModelInfo<TState> {
    pub name: &'static str,
    pub list_view_route: String,
    pub build: Box<dyn FnOnce(Vec<ModelViewInfo>, Router<TState>) -> Router<TState>>,
}

pub struct AdminRouter<TState: Send + Sync + Clone + 'static> {
    router: Router<TState>,
    models: Vec<ModelInfo<TState>>,
}

pub trait DieselState: Send + Sync + Clone + 'static {
    type TConnection: Connection;
    // fn get_connection() -> PgConnection<Backend = Pg>;
    fn get_connection(&self) -> Self::TConnection;
}

impl<T: DieselState> DieselState for Arc<T> {
    type TConnection = T::TConnection;

    fn get_connection(&self) -> Self::TConnection {
        self.as_ref().get_connection()
    }
}

impl<TState: Send + Sync + Clone + 'static> AdminRouter<TState> {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            models: Vec::new(),
        }
    }

    fn _register<
        TModel: AdminModel,
        IdType: Send + 'static + DeserializeOwned,
        TInsertionModel: AdminInputModel,
        TCreateMethod: FnOnce(State<TState>, TInsertionModel) -> TCreateOutput + Send + Sync + Clone + 'static,
        TGetMethod: FnOnce(State<TState>, IdType) -> TGetOutput + Send + Sync + Clone + 'static,
        TCreateOutput: Future<Output = ()> + Send,
        TListMethod: FnOnce(State<TState>) -> TListOutput + Send + Sync + Clone + 'static,
        TListOutput: Future<Output = Vec<TModel>> + Send,
        TGetOutput: Future<Output = TModel> + Send,
    >(
        mut self,
        create_method: TCreateMethod,
        list_method: TListMethod,
        get_method: TGetMethod,
        model_display_name: &'static str,
    ) -> Self {
        // TODO implemeneted get api
        let list_url = format!("/admin/{}", model_display_name);
        let create_url = format!("{}/create", list_url);

        let build = {
            let list_url = list_url.clone();
            let create_url = create_url.clone();

            move |models: Vec<ModelViewInfo>, router: Router<TState>| {
                let input_field_defs: Vec<FieldDef> = TInsertionModel::get_field_definitions();
                let list_field_defs: Vec<FieldDef> = TModel::get_field_definitions();

                let create_api_route =
                    |state: State<TState>, Form(input): Form<TInsertionModel>| async move {
                        create_method(state, input).await;

                        let mut headers = HeaderMap::new();
                        headers.insert("HX-Redirect", list_url.parse().unwrap());
                        (
                            StatusCode::CREATED,
                            headers,
                            // axum::extract::Json(output)
                        )
                    };

                let create_view_route = {
                    let models = models.clone();
                    let create_url = create_url.clone();
                    let fields = input_field_defs.clone();
                    || async move {
                        let template = AdminCreateTemplate {
                            fields,
                            create_endpoint: create_url.to_string(),
                            models,
                        };
                        HtmlTemplate(template)
                    }
                };

                let list_view_route = {
                    let models = models.clone();
                    |state: State<TState>| async move {
                        let values = list_method(state).await;
                        let fields = list_field_defs.clone();
                        let template = AdminListTemplate {
                            columns: fields,
                            rows: values.iter().map(|value| value.get_row()).collect(),
                            create_view_route: create_url.to_string(),
                            models,
                        };
                        HtmlTemplate(template)
                    }
                };

                let get_view_route = {
                    let models = models.clone();
                    |Path(id): Path<IdType>, state: State<TState>| async move {
                        let value = get_method(state, id).await;

                        let template = AdminViewTemplate {
                            models,
                            fields: value.get_field_values(),
                        };
                        HtmlTemplate(template)
                    }
                };

                let nested_router = Router::<TState>::new()
                    .route("/create", get(create_view_route))
                    .route("/create", post(create_api_route))
                    .route("/", get(list_view_route))
                    .route("/:id", get(get_view_route));

                router.nest(format!("/{}", model_display_name).as_str(), nested_router)
            }
        };

        println!("Registering {} at {}", model_display_name, list_url);
        self.models.push(ModelInfo {
            name: model_display_name,
            list_view_route: list_url.to_string(),
            build: Box::new(build),
        });

        self
    }

    // TODO Allow overriding some things
    // - path
    // - model name
    // probably not at register point though... Probably on the model with macro attributes
    pub fn register<
        TModel: RazerModel<TState, TInsertionModel> + 'static,
        TInsertionModel: AdminInputModel,
    >(
        self,
    ) -> Self {
        self._register(
            TModel::create_value,
            TModel::list_values,
            TModel::get_value,
            TModel::model_name(),
        )
    }

    pub fn build(mut self) -> Router<TState> {
        let models_view_info: Vec<ModelViewInfo> = self
            .models
            .iter()
            .map(|model| ModelViewInfo {
                name: model.name,
                list_view_route: model.list_view_route.clone(),
            })
            .collect();

        self.router = self.models.into_iter().fold(self.router, |router, model| {
            (model.build)(models_view_info.clone(), router)
        });

        self.router.route(
            "/",
            get(|| async {
                let template = AdminTemplate {
                    models: models_view_info,
                };
               HtmlTemplate(template)
            }),
        )
    }
}

type ModelId<'ident, TModel> = <<&'ident TModel as Identifiable>::Id as Deref>::Target;

impl<TConnection: Connection, TState: DieselState<TConnection = TConnection>> AdminRouter<TState> {
    pub fn register_diesel_model<'query, 'ident, TModel: AdminModel, TInsertable: AdminInputModel>(
        self,
    ) -> Self
    where
        'ident: 'static,
        Self: Sized,
        TConnection: LoadConnection,
        TInsertable: Insertable<TModel::Table>,
        TModel: HasTable + SelectableHelper<TConnection::Backend> + 'ident,
        &'ident TModel: Identifiable,
        // TODO Have to make hander happy!
        <&'ident TModel as Identifiable>::Id: Send + Deref + Clone,
        // <<&'ident TModel as Identifiable>::Id as ToOwned>::Owned: DeserializeOwned,
        // <&'ident TModel as Identifiable>::Id: Send + Deref,
        // ModelId<'ident, TModel>: Deserialize<'ident> + Send,
        for <'de> <<&'ident TModel as Identifiable>::Id as Deref>::Target: serde::Deserialize<'de> + Send,
        // for <'de> <&'ident TModel as Identifiable>::Id: serde::Deserialize<'de>,
        TModel::SelectExpression: QueryId,
        TModel::Table: FindDsl<<<&'ident TModel as Identifiable>::Id as Deref>::Target>,
        TModel::Table: SelectDsl<AsSelect<TModel, TConnection::Backend>> + LimitDsl,
        Select<TModel::Table, AsSelect<TModel, TConnection::Backend>>: LimitDsl,
        Limit<Select<TModel::Table, AsSelect<TModel, TConnection::Backend>>>:
            LoadQuery<'query, TConnection, TModel>,

        // <TModel::Table as FindDsl<
        //     <<&'ident TModel as Identifiable>::Id as Deref>::Target
        // >>::Output: SelectDsl<AsSelect<TModel, TConnection::Backend>>,

        // Get value
        <<TModel as HasTable>::Table as FindDsl<<<&'ident TModel as Identifiable>::Id as Deref>::Target>>::Output:
            SelectDsl<AsSelect<TModel, TConnection::Backend>>,
        Select<
            <<TModel as HasTable>::Table as FindDsl<<<&'ident TModel as Identifiable>::Id as Deref>::Target>>::Output,
            AsSelect<TModel, TConnection::Backend>,
        >: RunQueryDsl<TConnection> + LimitDsl,
        Limit<Select<
            <<TModel as HasTable>::Table as FindDsl<<<&'ident TModel as Identifiable>::Id as Deref>::Target>>::Output,
            AsSelect<TModel, TConnection::Backend>,
        >>: LoadQuery<'query, TConnection, TModel>,

        // Insert
        InsertStatement<TModel::Table, TInsertable::Values>: ExecuteDsl<TConnection>,
    {
        let create_value = |state: State<TState>, input: TInsertable| async move {
            let connection = &mut state.get_connection();
            println!("Inserting into {}", TModel::model_name());
            diesel::insert_into(TModel::table())
                .values(input)
                .execute(connection)
                .expect("Failed insert");
        };

        let list_values = |state: State<TState>| async move {
            let connection = &mut state.get_connection();
            let query = SelectDsl::select(TModel::table(), TModel::as_select());
            // TODO Don't limit by 100 / add paginiation to list
            LimitDsl::limit(query, 100)
                .load(connection)
                .expect("Failed query")
        };

        let get_value = |state: State<TState>, id: <<&'ident TModel as Identifiable>::Id as Deref>::Target | async move {
            let connection = &mut state.get_connection();
            let query = FindDsl::find(TModel::table(), id);
            let query = SelectDsl::select(query, TModel::as_select());
            let result = RunQueryDsl::first(query, connection).optional();
            result.unwrap().unwrap()
        };

        self._register(create_value, list_values, get_value, TModel::model_name())
    }
}
