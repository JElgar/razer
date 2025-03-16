mod error;
mod templates;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use axum::{
    Router,
    routing::{get, post, delete},
    response::IntoResponse,
    extract::{State, Path, Json},
};
use askama::Template;
use askama_axum::IntoResponse as AskamaIntoResponse;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::collections::HashMap;
use thiserror::Error;
pub use error::AdminError;
use serde_json::Value;
use templates::{ListTemplate, DashboardTemplate};

#[derive(Clone)]
pub struct ResourceInfo {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct ListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub filters: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct AdminField {
    pub name: String,
    pub field_type: FieldType,
    pub options: FieldOptions,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    Text,
    Number,
    Boolean,
    DateTime,
    Password,
    RichText,
    Select(Vec<SelectOption>),
    MultiSelect(Vec<SelectOption>),
    File,
    Image,
}

#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Default)]
pub struct FieldOptions {
    pub required: bool,
    pub searchable: bool,
    pub sortable: bool,
    pub sensitive: bool,
    pub help_text: Option<String>,
    pub placeholder: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary_color: "#3b82f6".to_string(),    // Blue
            secondary_color: "#6b7280".to_string(),   // Gray
            background_color: "#f3f4f6".to_string(), // Light gray
        }
    }
}

pub struct Admin {
    resources: Vec<ResourceInfo>,
    title: String,
    theme: Theme,
    models: HashMap<String, Box<dyn AdminModelRegistry>>,
}

impl Admin {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            title: "Admin Panel".to_string(),
            theme: Theme::default(),
            models: HashMap::new(),
        }
    }

    pub fn register<T: AdminModel + 'static>(mut self) -> Self {
        let path = T::admin_path().to_string();
        self.resources.push(ResourceInfo {
            name: T::admin_name().to_string(),
            path: path.clone(),
        });
        self.models.insert(path, Box::new(AdminModelRegistryImpl::<T>::new()));
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn router(self) -> Router {
        let state = Arc::new(AdminState {
            title: self.title,
            resources: self.resources,
            theme: self.theme,
            models: self.models,
        });

        Router::new()
            .route("/", get(dashboard_handler))
            .route("/:resource", get(list_handler))
            .route("/:resource", post(create_handler))
            .route("/:resource/:id", get(show_handler))
            .route("/:resource/:id", post(update_handler))
            .route("/:resource/:id", delete(delete_handler))
            .with_state(state)
    }
}

struct AdminState {
    title: String,
    resources: Vec<ResourceInfo>,
    theme: Theme,
    models: HashMap<String, Box<dyn AdminModelRegistry>>,
}

#[async_trait]
trait AdminModelRegistry: Send + Sync {
    async fn list_items(&self, params: ListParams) -> Result<Vec<Value>, AdminError>;
    async fn get_item(&self, id: i32) -> Result<Value, AdminError>;
    async fn create_item(&self, data: Value) -> Result<Value, AdminError>;
    async fn update_item(&self, id: i32, data: Value) -> Result<Value, AdminError>;
    async fn delete_item(&self, id: i32) -> Result<(), AdminError>;
    fn name(&self) -> String;
    fn list_fields(&self) -> Vec<String>;
}

struct AdminModelRegistryImpl<T: AdminModel> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AdminModel> AdminModelRegistryImpl<T> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T: AdminModel> AdminModelRegistry for AdminModelRegistryImpl<T> {
    async fn list_items(&self, params: ListParams) -> Result<Vec<Value>, AdminError> {
        let items = T::find_all(params).await?;
        Ok(items.into_iter().map(|item| serde_json::to_value(item).unwrap()).collect())
    }

    async fn get_item(&self, id: i32) -> Result<Value, AdminError> {
        let item = T::find_by_id(id).await?;
        Ok(serde_json::to_value(item).unwrap())
    }

    async fn create_item(&self, data: Value) -> Result<Value, AdminError> {
        let item: T = serde_json::from_value(data)?;
        let created = T::create(item).await?;
        Ok(serde_json::to_value(created).unwrap())
    }

    async fn update_item(&self, id: i32, data: Value) -> Result<Value, AdminError> {
        let item: T = serde_json::from_value(data)?;
        let updated = T::update(id, item).await?;
        Ok(serde_json::to_value(updated).unwrap())
    }

    async fn delete_item(&self, id: i32) -> Result<(), AdminError> {
        T::delete(id).await
    }

    fn name(&self) -> String {
        T::admin_name().to_string()
    }

    fn list_fields(&self) -> Vec<String> {
        T::list_fields()
    }
}

async fn dashboard_handler(
    State(state): State<Arc<AdminState>>,
) -> impl IntoResponse {
    let template = DashboardTemplate {
        title: state.title.clone(),
        resources: state.resources.clone(),
    };
    template
}

async fn list_handler(
    State(state): State<Arc<AdminState>>,
    Path(resource): Path<String>,
) -> Result<impl IntoResponse, AdminError> {
    println!("Debug - list_handler called for resource: {}", resource);
    let registry = state.models.get(&resource)
        .ok_or_else(|| AdminError::NotFound(format!("Resource {} not found", resource)))?;

    let params = ListParams::default();
    let items = registry.list_items(params).await?;
    println!("Debug - Items: {:#?}", items);
    
    let fields = registry.list_fields();
    println!("Debug - Fields: {:#?}", fields);
    println!("Debug - First item fields available: {:#?}", items.first().map(|item| item.as_object().map(|obj| obj.keys().collect::<Vec<_>>())));
    println!("Debug - Resource name: {}", registry.name());

    let template = ListTemplate {
        title: state.title.clone(),
        resource_name: registry.name(),
        resource_path: resource,
        fields,
        items,
    };

    Ok(template.into_response())
}

async fn create_handler(
    State(state): State<Arc<AdminState>>,
    Path(resource): Path<String>,
    Json(data): Json<Value>,
) -> Result<impl IntoResponse, AdminError> {
    let registry = state.models.get(&resource)
        .ok_or_else(|| AdminError::NotFound(format!("Resource {} not found", resource)))?;

    let item = registry.create_item(data).await?;
    Ok(Json(item))
}

async fn show_handler(
    State(state): State<Arc<AdminState>>,
    Path((resource, id)): Path<(String, i32)>,
) -> Result<impl IntoResponse, AdminError> {
    let registry = state.models.get(&resource)
        .ok_or_else(|| AdminError::NotFound(format!("Resource {} not found", resource)))?;

    let item = registry.get_item(id).await?;
    Ok(Json(item))
}

async fn update_handler(
    State(state): State<Arc<AdminState>>,
    Path((resource, id)): Path<(String, i32)>,
    Json(data): Json<Value>,
) -> Result<impl IntoResponse, AdminError> {
    let registry = state.models.get(&resource)
        .ok_or_else(|| AdminError::NotFound(format!("Resource {} not found", resource)))?;

    let item = registry.update_item(id, data).await?;
    Ok(Json(item))
}

async fn delete_handler(
    State(state): State<Arc<AdminState>>,
    Path((resource, id)): Path<(String, i32)>,
) -> Result<impl IntoResponse, AdminError> {
    let registry = state.models.get(&resource)
        .ok_or_else(|| AdminError::NotFound(format!("Resource {} not found", resource)))?;

    registry.delete_item(id).await?;
    Ok(())
}

pub trait AdminResource: Send + Sync {
    fn name(&self) -> &str;
    fn path(&self) -> &str;
    fn routes(&self) -> Router;
}

/// The main trait that needs to be implemented for a type to be used in the admin panel
#[async_trait]
pub trait AdminModel: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// The display name of this resource in the admin panel
    fn admin_name() -> &'static str;
    
    /// The URL path segment for this resource
    fn admin_path() -> &'static str;
    
    /// Fields to display in the list view
    fn list_fields() -> Vec<String>;
    
    /// Fields to display in the create form
    fn create_fields() -> Vec<AdminField>;
    
    /// Fields to display in the edit form
    fn edit_fields() -> Vec<AdminField>;
    
    /// Fields that can be searched
    fn search_fields() -> Vec<String>;
    
    /// Fields that can be used for filtering
    fn filter_fields() -> Vec<String>;
    
    /// Find a single resource by ID
    async fn find_by_id(id: i32) -> Result<Self, AdminError>;
    
    /// List resources with optional filtering and pagination
    async fn find_all(params: ListParams) -> Result<Vec<Self>, AdminError>;
    
    /// Create a new resource
    async fn create(data: Self) -> Result<Self, AdminError>;
    
    /// Update an existing resource
    async fn update(id: i32, data: Self) -> Result<Self, AdminError>;
    
    /// Delete a resource
    async fn delete(id: i32) -> Result<(), AdminError>;
    
    /// Hook called before creating a resource
    async fn before_create(&mut self) -> Result<(), AdminError> {
        Ok(())
    }
    
    /// Hook called after creating a resource
    async fn after_create(&mut self) -> Result<(), AdminError> {
        Ok(())
    }
    
    /// Hook called before updating a resource
    async fn before_update(&mut self) -> Result<(), AdminError> {
        Ok(())
    }
    
    /// Hook called after updating a resource
    async fn after_update(&mut self) -> Result<(), AdminError> {
        Ok(())
    }
    
    /// Hook called before deleting a resource
    async fn before_delete(&mut self) -> Result<(), AdminError> {
        Ok(())
    }
    
    /// Hook called after deleting a resource
    async fn after_delete(&mut self) -> Result<(), AdminError> {
        Ok(())
    }
}
