use axum::Router;
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use razer_core::{Admin, AdminError, FieldConfig, Resource, ValidationResult};
use razer_ui::render_toggle_widget;
use razer_core_derive::AdminResource;

#[derive(Clone)]
struct AdminContext {
    my_models: Arc<Mutex<Vec<MyModel>>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, AdminResource)]
#[admin(name = "abc")]
struct MyModel {
    #[admin(readonly)]
    id: i32,
    name: String,
    is_adult: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct MyModelInput {
    name: String,
    is_adult: bool,
}

#[tokio::main]
async fn main() {
    let resource = Resource::<AdminContext, i32, MyModel, MyModelInput> {
        name: "My Model".to_string(),
        path: "model".to_string(),
        id_field_id: "id".to_string(),

        list_items: Arc::new(|context| {
            Box::pin(async move {
                let models = context.my_models.lock().await;
                Ok(models.iter().cloned().collect())
            })
        }),

        get_item: Arc::new(|context, id| {
            Box::pin(async move {
                let models = context.my_models.lock().await;
                let item = models.iter().find(|item| item.id == id);
                match item {
                    Some(value) => Ok(value.clone()),
                    None => Err(AdminError::NotFound),
                }
            })
        }),

        create_item: Arc::new(|context, data| {
            Box::pin(async move {
                let next_id = {
                    let models = context.my_models.lock().await;
                    models.iter().max_by_key(|item| item.id).map_or(0, |it| it.id+ 1)
                };

                let item = MyModel {
                    id: next_id,
                    is_adult: data.is_adult,
                    name: data.name,
                };

                let mut models = context.my_models.lock().await;
                models.push(item.clone());

                Ok(item)
            })
        }),
        // TODO Update this to generate a field config which is a struct from field name to field
        // config. This allows users to update with
        // field_configs: FieldConfig {
        //  some_field: StringFieldConfig {
        //    render: some_override,
        //    ..MyModel::default_field_configs()
        //  },
        //  ..MyModel::default_field_configs()
        // }
        field_configs: MyModel::default_field_configs(),

        // field_configs: vec![
        //     FieldConfig::create_number_config("id".to_string(), "Id".to_string(), true),
        //     FieldConfig::create_text_config("name".to_string(), "Name".to_string(), false),
        //     FieldConfig::create_boolean_config("is_adult".to_string(), "Is adult".to_string(), false),
        //     // FieldConfig {
        //     //     field_id: "is_adult".to_string(),
        //     //     display_name: "Is an adult?".to_string(),
        //     //     help_text: None,
        //     //     description: None,
        //     //     render: Arc::new(move |value| {
        //     //         // TODO Create functions to render with return type string inside razer_ui e.g.
        //     //         // render_toggle
        //     //         Ok(render_toggle_widget(
        //     //             "is_adult".to_string(),
        //     //             "Is an adult?".to_string(),
        //     //             value,
        //     //         ))
        //     //     }),
        //     //     validate: Arc::new(|value| ValidationResult::Valid),
        //     // },
        // ],
    };

    let admin = Admin::new(AdminContext {
        my_models: Arc::new(Mutex::new(vec![MyModel {
            id: 1,
            name: "Susan".to_string(),
            is_adult: true,
        }])),
    })
    .register(resource);

    let app = Router::new().nest("/admin", razer_axum::AxumRouter(admin).into());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
