use askama::Template;
use axum::{
    extract::{Path, RawForm},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use razer_core::Admin;
use razer_ui::{
    get_default_template_css, render_create_resource_view, render_list_resource_view,
    render_not_found_view, render_view_resource_view, AdminListTemplateRow,
};

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

pub struct AxumRouter<TContext>(pub Admin<TContext>);

impl<TContext: Send + Sync + Clone + 'static> Into<Router> for AxumRouter<TContext> {
    fn into(self) -> Router {
        self.0
            .resources
            .into_iter()
            .fold(Router::new(), |router, resource| {
                let list_url = format!("/admin/{}", resource.path);
                let context = self.0.context.clone();

                let get_view_route = {
                    let resource_name = resource.name.clone();
                    let field_configs = resource.field_configs.clone();
                    let context = context.clone();

                    |Path(id): Path<String>| async move {
                        // TODO no unwrap
                        let value = (*resource.get_item)(context, id).await;

                        let html = match value {
                            Ok(value) => render_view_resource_view(
                                resource_name.clone(),
                                field_configs
                                    .iter()
                                    .map(|field| {
                                        let field_value = value.get(&field.field_id).unwrap();
                                        // TODO Handle
                                        (field.render)(field_value.clone()).unwrap()
                                    })
                                    .collect(),
                            ),
                            Err(razer_core::AdminError::NotFound) => render_not_found_view(),
                            Err(razer_core::AdminError::InternalError) => {
                                todo!()
                            }
                        };

                        axum::response::Html(html).into_response()
                    }
                };

                let list_view_route = {
                    let resource_name = resource.name.clone();
                    let resource_path = resource.path.clone();
                    let field_configs = resource.field_configs.clone();
                    let headers = resource
                        .field_configs
                        .iter()
                        .map(|config| config.display_name.clone())
                        .collect();
                    let context = context.clone();

                    || async move {
                        let items = (*resource.list_items)(context).await;

                        let html = match items {
                            Ok(items) => {
                                let values = items
                                    .iter()
                                    .map(|item| {
                                        let item_id = item.get(&resource.id_field_id).unwrap().clone();

                                        AdminListTemplateRow {
                                            item_link: format!("/admin/{}/{}", resource_path, item_id),
                                            data: field_configs
                                                .iter()
                                                // TODO Render more than just strings - have render
                                                // function for list view?
                                                .map(|config| {
                                                    item.get(&config.field_id).unwrap().to_string()
                                                })
                                                .collect(),
                                        }
                                    })
                                    .collect();

                                render_list_resource_view(
                                    resource_name.clone(),
                                    // TODO Get base url from somewhere
                                    format!("/admin/{}/create", resource_path),
                                    headers,
                                    values,
                                )
                            }
                            Err(_) => todo!(),
                        };

                        axum::response::Html(html).into_response()
                    }
                };

                let create_view_route = {
                    let resource_path = resource.path.clone();

                    || async move {
                        // TODO This should not be in razer package!!
                        let html = render_create_resource_view(
                            resource.name.clone(),
                            // TODO Get base url from somewhere
                            format!("/admin/{}/create", resource_path),
                            resource
                                .field_configs
                                .iter()
                                .filter(|field_config| field_config.create_config.is_some())
                                .map(|field| {
                                    // TODO Don't unwrap - render error page
                                    (field.render)(serde_json::Value::Null).unwrap()
                                })
                                .collect(),
                        );

                        axum::response::Html(html).into_response()
                    }
                };

                let create_api_route = {
                    let context = context.clone();

                    // |Form(input): Form<serde_json::Map<String, serde_json::Value>>| async move {
                    |RawForm(bytes): RawForm| async move {
                        let mut headers = HeaderMap::new();

                        let created_item = (*resource.create_item)(context, &bytes).await;
                        dbg!(created_item);

                        headers.insert("HX-Redirect", list_url.parse().unwrap());

                        (
                            StatusCode::CREATED,
                            headers,
                            // axum::extract::Json(output)
                        )
                    }
                };

                let nested_router = Router::new()
                    .route("/create", get(create_view_route))
                    .route("/create", post(create_api_route))
                    .route("/", get(list_view_route))
                    .route("/{id}", get(get_view_route));

                router.nest(format!("/{}", resource.path).as_str(), nested_router)
            })
            .route(
                "/assets/admin.css",
                get(async || {
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "text/css")],
                        get_default_template_css(),
                    )
                }),
            )
    }
}
