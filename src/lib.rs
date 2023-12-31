use std::{pin::Pin, future::Future, fmt::format, sync::Arc};

use axum::{http::{StatusCode, HeaderMap}, extract::{State, self}, Form, response::IntoResponse, Router, routing::{post, get}};
use askama::Template;
use diesel::{Insertable, PgConnection, Table, Queryable, Connection, backend::Backend, pg::Pg, connection::AnsiTransactionManager, QueryDsl, query_builder::{Query, AsQuery, QueryId, InsertStatement, ReturningClause}, query_dsl::methods::{FilterDsl, SelectDsl, LoadQuery, LimitDsl, ExecuteDsl}, Expression, Selectable, SelectableHelper, helper_types::{AsSelect, Select, Limit}, BelongingToDsl, associations::HasTable, QuerySource, SelectableExpression, expression::{ValidGrouping, MixedAggregates}, Identifiable};
use serde::de::DeserializeOwned;
use uuid::Uuid;

pub type TableHeaderData = &'static str;
pub type TableCellData = String;
pub trait TableDataType {
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
    models: Vec<ModelViewInfo>
}

#[derive(Template)]
#[template(path="admin-list.html")]
pub struct AdminListTemplate {
    models: Vec<ModelViewInfo>,
    headers: Vec<&'static str>,
    rows: Vec<Vec<String>>,
    create_view_route: String,
}

#[derive(Template)]
#[template(path="admin-create.html")]
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
                format!("Error rendering template: {}", err)
            ).into_response(),
        }
    }
}

pub trait AdminInputModel: serde::Serialize + DeserializeOwned + Send + Sync + 'static {
    fn get_field_definitions() -> Vec<FieldDef>;
}

pub trait AdminModel: serde::Serialize + TableDataType {
    fn model_name() -> &'static str;

    // Model and insertion model should be different traits
    // TODO Readonly stuff should be in a different trait, implemented for the 
    // TODO Delete these
    // fn get_list_url() -> String {
    //     format!("/admin/{}/list", Self::model_name())
    // }

    // fn get_create_url() -> String {
    //     format!("/admin/{}/create", Self::model_name())
    // }
}

#[async_trait::async_trait]
pub trait RazerModel<
    AppState: Clone + Send + Sync + 'static,
    InsertionModel: AdminInputModel,
>: AdminModel {
    async fn list_values(
        state: State<AppState>,
    ) -> Vec<Self> where Self: Sized;

    async fn create_value(
        state: State<AppState>,
        input: InsertionModel,
    );
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

pub struct AdminRouter<TState> {
    router: Router<TState>,
    models: Vec<ModelInfo<TState>>,
}

pub trait DieselState: Send + Sync + Clone + 'static {
    type TConnection: Connection;
    // fn get_connection() -> PgConnection<Backend = Pg>;
    fn get_connection(&self) -> Self::TConnection;
}

// #[async_trait::async_trait]
// impl <
//     TConnectionST,
//     TConnectionBackend: Backend,
//     TConnection: Connection<Backend = TConnectionBackend>,
//     TModel: AdminModel + Queryable<TConnectionST, TConnectionBackend>,
//     TInsertable: Insertable<TModel> + AdminInputModel,
//     TState: DieselState<TConnection>,
//     TTable: Table
// > RazerModel<TState, TInsertable> for DieselAdminModel<TTable> {
//     async fn list_values(
//         axum::extract::State(state): axum::extract::State<TState>,
//     ) -> Vec<Self> {
//         let connection = &mut establish_connection();
//         let results = my_models
//             .filter(published.eq(true))
//             .limit(5)
//             .select(MyDieselModel::as_select())
//             .load(connection)
//             .expect("Error loading posts");
//     }
// 
//     async fn create_value(
//         axum::extract::State(state): axum::extract::State<TState>,
//         input: TInsertable,
//     ) -> Self {
//         todo!()
//     }
// }

// impl <TState, TInsertionModel> RazerModel<TState, TInsertionModel> for TModel where TInsertionModel: AdminInputModel, TModel: AdminModel {
//     async fn list_values() -> Vec<Self> {
//         todo!()
//     }
// 
//     async fn create_value(input: TInsertionModel) -> Self {
//         todo!()
//     }
// }
// 
// impl <TState: DieselState> AdminRouter<TState> {
//     pub fn register_diesel_model<TModel, TInsertable, TTable>(mut self, table: TTable) -> Self where TInsertable: Insertable<T> {
//         impl RazerModel<TState, TInsertable> for TModel {
//             fn list_values<'async_trait>(state:State<TState> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = Vec<Self> > + core::marker::Send+'async_trait> >where Self:'async_trait {
//                 todo!()
//             }
//         }
// 
//         let build = {
//             |models: Vec<ModelViewInfo>, router: Router<TState>| {
//                 async fn create_api_route<
//                     TState: DieselState,
//                 > (
//                     state: State<TState>,
//                     Form(input): Form<TModel>,
//                 ) -> (
//                     StatusCode,
//                     HeaderMap,
//                     extract::Json<TModel>
//                 ) {
//                     diesel::insert_into(my_models::table)
//                         .values(&new_post)
//                         .returning(MyDieselModel::as_returning())
//                         .get_result(conn)
//                         .expect("Error saving new post")
// 
//                     let mut headers = HeaderMap::new();
//                     headers.insert("HX-Redirect", TModel::get_list_url().parse().unwrap());
//                     (
//                         StatusCode::CREATED,
//                         headers,
//                         axum::extract::Json(input)
//                     )
//                 }
// 
//                 let create_view_route = {
//                     let models = models.clone();
//                     || async {
//                         let template = AdminCreateTemplate {
//                             fields: TModel::get_field_definitions(),
//                             create_endpoint: TModel::get_create_url(),
//                             models,
//                         };
//                         HtmlTemplate(template)
//                     }
//                 };
// 
//                 let list_view_route = |state: State<TState>| async {
//                     let values = TModel::list_values(state).await;
//                     let template = AdminListTemplate {
//                         headers: TModel::get_headers(),
//                         rows: values.iter().map(|value| value.get_row()).collect(),
//                         create_view_route: TModel::get_create_url(),
//                         models,
//                     };
//                     HtmlTemplate(template)
//                 };
// 
//                 let nested_router = Router::<TState>::new()
//                     .route("/create", get(create_view_route))
//                     .route("/create", post(create_api_route::<TState, TModel>))
//                     .route("/list", get(list_view_route));
// 
//                 router.nest(format!("/{}", TModel::model_name()).as_str(), nested_router)
// 
//             }
//         };
// 
//         self.models.push(ModelInfo {
//             name: TModel::model_name(),
//             list_view_route: TModel::get_list_url(),
//             build: Box::new(build),
//         });
// 
//         self
//     }
// }

impl <TState: Send + Sync + Clone + 'static> AdminRouter<TState> {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            models: Vec::new(),
        }
    }

    fn _register<
        TModel: serde::Serialize + TableDataType,
        TInsertionModel: AdminInputModel,
        TCreateMethod: FnOnce(State<TState>, TInsertionModel) -> TCreateOutput + Send + Sync + Clone + 'static,
        TCreateOutput: Future<Output = ()> + Send,
        TListMethod: FnOnce(State<TState>) -> TListOutput + Send + Sync + Clone + 'static,
        TListOutput: Future<Output = Vec<TModel>> + Send,
    >(
        mut self,
        create_method: TCreateMethod,
        list_method: TListMethod,
        model_display_name: &'static str,
    ) -> Self {
        let list_url = format!("/admin/{}", model_display_name);
        let create_url = format!("{}/create", list_url);

        let build = {
            let list_url = list_url.clone();
            let create_url = create_url.clone();

            move |models: Vec<ModelViewInfo>, router: Router<TState>| {
                let create_api_route = |state: State<TState>, Form(input): Form<TInsertionModel>| async move {
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
                    || async move {
                        let template = AdminCreateTemplate {
                            fields: TInsertionModel::get_field_definitions(),
                            create_endpoint: create_url.to_string(),
                            models,
                        };
                        HtmlTemplate(template)
                    }
                };

                let list_view_route = |state: State<TState>| async move {
                    let values = list_method(state).await;
                    let template = AdminListTemplate {
                        headers: TModel::get_headers(),
                        rows: values.iter().map(|value| value.get_row()).collect(),
                        create_view_route: create_url.to_string(),
                        models,
                    };
                    HtmlTemplate(template)
                };

                let nested_router = Router::<TState>::new()
                    .route("/create", get(create_view_route))
                    .route("/create", post(create_api_route))
                    .route("/", get(list_view_route));

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
    >(self) -> Self {
        self._register(
            TModel::create_value,
            TModel::list_values,
            TModel::model_name(),
        )
    }

    pub fn build(mut self) -> Router<TState> {
        let models_view_info: Vec<ModelViewInfo> = self.models
            .iter()
            .map(|model| ModelViewInfo {
                name: model.name,
                list_view_route: model.list_view_route.clone()
            })
            .collect();

        self.router = self.models.into_iter().fold(self.router, |router, model| {
            (model.build)(models_view_info.clone(), router)
        });

        self.router.route("/", get(|| async {
            let template = AdminTemplate {
                models: models_view_info,
            };
            HtmlTemplate(template)
        }))
    }
}

// struct TestModel;
// impl TestModel {
//     fn internal_get_by_id<
//         TTable,
//         C,
//         TConnection: Connection,
//     >(
//         diesel_table: TTable,
//         table_id: C,     
//         conn: &TConnection,
//         id: Uuid,
//     ) -> Option<Self>
//     where
//         Self: Sized,
//         TTable: Table + FilterDsl<diesel::dsl::Eq<C, Uuid>>,
//         C: diesel::Column + Expression<SqlType = diesel::sql_types::Uuid>,
//         diesel::dsl::Filter<TTable, diesel::dsl::Eq<C, Uuid>>: LimitDsl,
//         diesel::dsl::Limit<diesel::dsl::Filter<TTable, diesel::dsl::Eq<C, Uuid>>>: LoadQuery<TConnection, Self>,
//         Self: Queryable<diesel::dsl::SqlTypeOf<diesel::dsl::Limit<diesel::dsl::Filter<T, diesel::dsl::Eq<C, Uuid>>>>, TConnection::Backend>,
//     {
//         diesel_table
//             .filter(table_id.eq(id))
//             .first(conn.raw())
//             .optional()
//     }
// }

// trait DieselAdminRouter<
//     TConnection: Connection,
//     TState: DieselState<TConnection = TConnection>,
// > {
//     fn register_diesel_model<
//         'query,
//         TModel: AdminModel,
//         TInsertable: AdminInputModel,
//     >(self) -> Self
//     where
//         Self: Sized,
//         TInsertable: Insertable<TModel>,
//         TModel: HasTable + SelectableHelper<TConnection::Backend>,
//         TModel::SelectExpression: QueryId,
//         TModel::Table: SelectDsl<AsSelect<TModel, TConnection::Backend>> + LimitDsl,
//         Select<TModel::Table, AsSelect<TModel, TConnection::Backend>>: LimitDsl + Table,
//         Limit<Select<TModel::Table, AsSelect<TModel, TConnection::Backend>>>: LoadQuery<'query, TConnection, TModel>;
// }

impl <TConnection: Connection, TState: DieselState<TConnection = TConnection>> AdminRouter<TState> {
    pub fn register_diesel_model<
        'query,
        TModel: AdminModel,
        TInsertable: AdminInputModel,
    >(self) -> Self where
        Self: Sized,
        TInsertable: Insertable<TModel::Table>,
        TModel: HasTable + SelectableHelper<TConnection::Backend>,
        TModel::SelectExpression: QueryId,
        TModel::Table: SelectDsl<AsSelect<TModel, TConnection::Backend>> + LimitDsl,
        Select<TModel::Table, AsSelect<TModel, TConnection::Backend>>: LimitDsl,
        Limit<Select<TModel::Table, AsSelect<TModel, TConnection::Backend>>>: LoadQuery<'query, TConnection, TModel>,

        // Insert
        InsertStatement<TModel::Table, TInsertable::Values>: ExecuteDsl<TConnection>,
    {
        use diesel::RunQueryDsl;

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
            let query = SelectDsl::select(
                TModel::table(), TModel::as_select()
            );
            return LimitDsl::limit(query, 10).load(connection).expect("Failed query");
        };

        self._register(
            create_value,
            list_values,
            TModel::model_name(),
        )
    }
}

impl<T: DieselState> DieselState for Arc<T> {
    type TConnection = T::TConnection;

    fn get_connection(&self) -> Self::TConnection {
        self.as_ref().get_connection()
    }
}
