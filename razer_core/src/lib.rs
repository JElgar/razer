use razer_ui::{render_checkbox_widget, render_number_input_widget, render_text_widget};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, future::Future, pin::Pin, str::FromStr, sync::Arc};

#[derive(Debug)]
pub enum AdminError {
    NotFound,
    InternalError,
}

// This should be part of razer (not core) as users will depend on it directly
pub struct Resource<TContext, TId: ToString, TItem: Serialize, TCreateInput: DeserializeOwned> {
    pub name: String,
    pub path: String,
    pub id_field_id: String,

    // TODO Can this be box again??
    pub list_items: Arc<
        dyn Fn(TContext) -> Pin<Box<dyn Future<Output = Result<Vec<TItem>, AdminError>> + Send>>
            + Send
            + Sync,
    >,
    pub get_item: Arc<
        dyn Fn(TContext, TId) -> Pin<Box<dyn Future<Output = Result<TItem, AdminError>> + Send>>
            + Send
            + Sync,
    >,
    pub create_item: Arc<
        dyn Fn(
                TContext,
                TCreateInput,
            ) -> Pin<Box<dyn Future<Output = Result<TItem, AdminError>> + Send>>
            + Send
            + Sync,
    >,

    // Field name to config
    pub field_configs: Vec<FieldConfig>,
}

pub struct JsonResource<TContext> {
    pub id_field_id: String,
    pub name: String,
    pub path: String,

    pub list_items: Arc<
        dyn Fn(
                TContext,
            )
                -> Pin<Box<dyn Future<Output = Result<Vec<JsonResourceItem>, AdminError>> + Send>>
            + Send
            + Sync,
    >,
    pub get_item: Arc<
        dyn Fn(
                TContext,
                String,
            )
                -> Pin<Box<dyn Future<Output = Result<JsonResourceItem, AdminError>> + Send>>
            + Send
            + Sync,
    >,
    pub create_item: Arc<
        dyn Fn(TContext, &[u8]) -> Pin<Box<dyn Future<Output = Result<(), AdminError>> + Send>>
            + Send
            + Sync,
    >,

    // Field name to config
    pub field_configs: Vec<FieldConfig>,
}

impl<
        TContext: Send + 'static,
        TId: ToString + FromStr + 'static,
        TItem: Serialize + 'static,
        TCreateInput: DeserializeOwned + 'static + Send,
    > From<Resource<TContext, TId, TItem, TCreateInput>> for JsonResource<TContext>
{
    fn from(value: Resource<TContext, TId, TItem, TCreateInput>) -> Self {
        let list_items_closure = value.list_items;
        let get_item_closure = value.get_item;
        let create_item_closure = value.create_item;
        let field_configs = value.field_configs;

        JsonResource {
            name: value.name,
            path: value.path,
            id_field_id: value.id_field_id,
            // TODO Having to clone field configs here is a bit sad - its because we have to move
            // field configs into the create closure. Would be nice to understand this and see if
            // there is an alternative
            field_configs: field_configs.clone(),
            list_items: Arc::new(move |context| {
                let list_items_closure = Arc::clone(&list_items_closure);

                Box::pin(async move {
                    let items = (list_items_closure)(context).await?;

                    let json_items = items
                        .into_iter()
                        .map(|item| {
                            serde_json::to_value(item)
                                .unwrap()
                                .as_object()
                                .unwrap()
                                .clone()
                        })
                        .collect::<Vec<_>>();
                    Ok(json_items)
                })
            }),
            get_item: Arc::new(move |context, id| {
                let get_item_closure = Arc::clone(&get_item_closure);

                Box::pin(async move {
                    let id = TId::from_str(&id).map_err(|_| AdminError::InternalError)?;
                    let item = (get_item_closure)(context, id).await?;
                    let value =
                        serde_json::to_value(item).map_err(|_| AdminError::InternalError)?;
                    match value {
                        serde_json::Value::Object(map) => Ok(map),
                        _ => Err(AdminError::NotFound),
                    }
                })
            }),
            create_item: Arc::new(move |context, data| {
                let create_item_closure = Arc::clone(&create_item_closure);
                // let item_to_insert: Result<TCreateInput, _> = serde_urlencoded::from_bytes(&data);

                let form_data = form_urlencoded::parse(&data);
                let form_data_map: HashMap<String, String> = form_data.into_owned().collect();
                dbg!(&form_data_map);

                let json_data =
                    field_configs
                        .iter()
                        .fold(serde_json::Map::new(), |mut map, field_config| {
                            let field_id = &field_config.field_id;

                            if let Some(create_config) = &field_config.create_config {
                                let field_value =
                                    (create_config.value_from_form_value)(form_data_map.get(field_id));
                                map.insert(field_id.clone(), field_value);
                            }

                            return map;
                        });

                dbg!(&json_data);

                // let deserializer = serde_urlencoded::Deserializer::new();
                // let parse_result: Result<TCreateInput, _> = serde_path_to_error::deserialize(deserializer);

                let parse_result = serde_json::from_value(serde_json::Value::Object(json_data));

                match parse_result {
                    Ok(item_to_insert) => Box::pin(async move {
                        (create_item_closure)(context, item_to_insert).await?;
                        Ok(())
                    }),
                    Err(e) => {
                        dbg!(e);
                        todo!()
                    }
                }

                // dbg!(item_to_insert);

                // // serde_urlencoded::Deserializer::new(form_urlencoded::parse(&data));

                // dbg!("Creating with: {}", &data);
            }),
        }
    }
}

type JsonResourceItem = serde_json::Map<String, serde_json::Value>;

pub struct Theme {}

impl Default for Theme {
    fn default() -> Self {
        Self {}
    }
}

pub struct Admin<TContext> {
    pub title: String,
    pub theme: Theme,
    pub resources: Vec<JsonResource<TContext>>,
    pub context: TContext,
}

impl<TContext: Send + 'static> Admin<TContext> {
    pub fn new(context: TContext) -> Self {
        Admin {
            title: "Razer admin".to_string(),
            theme: Theme::default(),
            resources: vec![],
            context,
        }
    }

    // TODO Is sta
    pub fn register<
        TId: ToString + FromStr + 'static,
        TItem: Serialize + 'static,
        TCreateInput: DeserializeOwned + 'static + Send,
    >(
        mut self,
        resource: Resource<TContext, TId, TItem, TCreateInput>,
    ) -> Self {
        self.resources.push(resource.into());
        return self;
    }
}

pub enum ValidationResult {
    Invalid(String),
    Valid,
}

#[derive(Clone)]
pub struct CreateConfig {
    pub validate: Arc<dyn Fn(serde_json::Value) -> ValidationResult + Send + Sync>,
    pub value_from_form_value: Arc<dyn Fn(Option<&String>) -> serde_json::Value + Send + Sync>,
}

#[derive(Clone)]
pub struct FieldConfig {
    pub field_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub help_text: Option<String>,
    pub render: Arc<dyn Fn(serde_json::Value) -> Result<String, AdminError> + Send + Sync>,

    pub create_config: Option<CreateConfig>,
}

impl FieldConfig {
    pub fn create_text_config(field_id: String, display_name: String, read_only: bool) -> Self {
        Self {
            field_id: field_id.clone(),
            display_name: display_name.clone(),
            help_text: None,
            description: None,
            render: Arc::new(move |value| {
                Ok(render_text_widget(
                    field_id.clone(),
                    display_name.clone(),
                    value,
                ))
            }),
            create_config: if read_only {
                None
            } else {
                Some(CreateConfig {
                    validate: Arc::new(|value| ValidationResult::Valid),
                    value_from_form_value: Arc::new(|value| {
                        serde_json::Value::String(value.unwrap().clone())
                    }),
                })
            },
        }
    }

    pub fn create_boolean_config(field_id: String, display_name: String, read_only: bool) -> Self {
        Self {
            field_id: field_id.clone(),
            display_name: display_name.clone(),
            help_text: None,
            description: None,
            render: Arc::new(move |value| {
                Ok(render_checkbox_widget(
                    field_id.clone(),
                    display_name.clone(),
                    value,
                ))
            }),
            create_config: if read_only {
                None
            } else {
                Some(CreateConfig {
                    validate: Arc::new(|value| ValidationResult::Valid),
                    value_from_form_value: Arc::new(|value| {
                        serde_json::Value::Bool(value.is_some())
                    }),
                })
            },
        }
    }

    pub fn create_number_config(field_id: String, display_name: String, read_only: bool) -> Self {
        Self {
            field_id: field_id.clone(),
            display_name: display_name.clone(),
            help_text: None,
            description: None,
            render: Arc::new(move |value| {
                Ok(render_number_input_widget(
                    field_id.clone(),
                    display_name.clone(),
                    value,
                ))
            }),
            create_config: if read_only {
                None
            } else {
                Some(CreateConfig {
                    validate: Arc::new(|value| ValidationResult::Valid),
                    value_from_form_value: Arc::new(|value| {
                        serde_json::Value::Number(value.unwrap().parse().unwrap())
                    }),
                })
            },
        }
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
