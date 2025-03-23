// #[derive(Template)]
// #[template(path = "admin_home.html")]
// struct AdminHomeTemplate {
//     page_title: String,
//     admin_title: String,
//     resources: Vec<JsonResource>,
// }

use askama::Template;

#[derive(Debug)]
pub enum RenderError {
    InternalError,
}

pub fn get_default_template_css() -> &'static str {
    return include_str!("../styles/output.css");
}

pub fn render_text_widget(
    field_id: String,
    display_name: String,
    value: serde_json::Value,
) -> String {
    TextAreaWidget {
        field_id: field_id.clone(),
        display_name: display_name.clone(),
        value: value.as_str().map(|val| val.to_string()),
    }
    .render()
    .unwrap()
}

pub fn render_checkbox_widget(
    field_id: String,
    display_name: String,
    value: serde_json::Value,
) -> String {
    CheckboxWidget {
        field_id: field_id.clone(),
        display_name: display_name.clone(),
        value: value.as_bool(),
    }
    .render()
    .unwrap()
}

pub fn render_toggle_widget(
    field_id: String,
    display_name: String,
    value: serde_json::Value,
) -> String {
    ToggleWidget {
        field_id: field_id.clone(),
        display_name: display_name.clone(),
        value: value.as_bool(),
    }
    .render()
    .unwrap()
}

// TODO Take in struct which implements default (or semi default because id will be required)
pub fn render_number_input_widget(
    field_id: String,
    display_name: String,
    value: serde_json::Value,
) -> String {
    NumberInputWidget {
        field_id: field_id.clone(),
        display_name: display_name.clone(),
        value: value.as_i64(),
    }
    .render()
    .unwrap()
}

pub struct AdminListTemplateRow {
    pub data: Vec<String>,
    pub item_link: String,
}

#[derive(Template)]
#[template(path = "admin_list.html")]
struct AdminListTemplate {
    page_title: String,
    create_view_endpoint: String,
    headers: Vec<String>,
    rows: Vec<AdminListTemplateRow>,
}

pub fn render_list_resource_view(
    resource_name: String,
    create_view_endpoint: String,
    // field_widgets: Vec<String>
    headers: Vec<String>,
    rows: Vec<AdminListTemplateRow>,
) -> String {
    AdminListTemplate {
        page_title: resource_name,
        create_view_endpoint,
        rows,
        headers,
        // fields: field_widgets,
    }
    .render()
    .unwrap()
}

pub fn render_view_resource_view(resource_name: String, field_widgets: Vec<String>) -> String {
    AdminViewTemplate {
        page_title: resource_name,
        fields: field_widgets,
    }
    .render()
    .unwrap()
}

#[derive(Template)]
#[template(path = "admin_view.html")]
struct AdminViewTemplate {
    page_title: String,
    fields: Vec<String>,
}

pub fn render_create_resource_view(
    resource_name: String,
    create_endpoint: String,
    field_widgets: Vec<String>
) -> String {
    AdminCreateTemplate {
        page_title: resource_name,
        create_endpoint,
        fields: field_widgets,
    }.render().unwrap()
}

#[derive(Template)]
#[template(path = "admin_create.html")]
struct AdminCreateTemplate {
    page_title: String,
    fields: Vec<String>,
    create_endpoint: String,
}

#[derive(Template)]
#[template(path = "widgets/text_area.html")]
struct TextAreaWidget {
    field_id: String,
    display_name: String,
    value: Option<String>,
}

#[derive(Template)]
#[template(path = "widgets/checkbox.html")]
struct CheckboxWidget {
    field_id: String,
    display_name: String,
    value: Option<bool>,
}

#[derive(Template)]
#[template(path = "widgets/toggle.html")]
struct ToggleWidget {
    field_id: String,
    display_name: String,
    value: Option<bool>,
}

#[derive(Template)]
#[template(path = "widgets/number_input.html")]
struct NumberInputWidget {
    field_id: String,
    display_name: String,
    value: Option<i64>,
}

pub fn render_not_found_view() -> String {
    NotFound {
        page_title: "Not found".to_string(),
    }
    .render()
    .unwrap()
}

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFound {
    page_title: String,
}
