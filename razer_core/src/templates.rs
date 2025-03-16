use askama::Template;
use serde_json::Value;
use crate::ResourceInfo;

#[derive(Template)]
#[template(path = "list.html")]
pub struct ListTemplate {
    pub title: String,
    pub resource_name: String,
    pub resource_path: String,
    pub fields: Vec<String>,
    pub items: Vec<Value>,
}

impl ListTemplate {
    fn get_field_value(&self, item: &Value, field: &str) -> String {
        item.get(field)
            .and_then(|v| match v {
                Value::String(s) => Some(s.to_string()),
                Value::Number(n) => Some(n.to_string()),
                Value::Bool(b) => Some(b.to_string()),
                Value::Null => Some(String::new()),
                Value::Array(a) => Some(format!("{:?}", a)),
                Value::Object(o) => Some(format!("{:?}", o)),
            })
            .unwrap_or_default()
    }
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub title: String,
    pub resources: Vec<ResourceInfo>,
} 