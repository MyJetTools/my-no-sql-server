const TABLE_KEY: &str = "mns_selected_table";
const PARTITION_KEY: &str = "mns_selected_partition";
const THEME_KEY: &str = "mns_theme";

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub fn save_selected_table(name: &str) {
    if let Some(s) = local_storage() {
        let _ = s.set_item(TABLE_KEY, name);
    }
}

pub fn load_selected_table() -> Option<String> {
    let s = local_storage()?;
    s.get_item(TABLE_KEY).ok()?.filter(|v| !v.is_empty())
}

pub fn save_selected_partition(pk: Option<&str>) {
    let Some(s) = local_storage() else {
        return;
    };
    match pk {
        Some(v) => {
            let _ = s.set_item(PARTITION_KEY, v);
        }
        None => {
            let _ = s.remove_item(PARTITION_KEY);
        }
    }
}

pub fn load_selected_partition() -> Option<String> {
    let s = local_storage()?;
    s.get_item(PARTITION_KEY).ok()?.filter(|v| !v.is_empty())
}

pub fn save_theme(theme: &str) {
    if let Some(s) = local_storage() {
        let _ = s.set_item(THEME_KEY, theme);
    }
}

pub fn load_theme() -> Option<String> {
    let s = local_storage()?;
    s.get_item(THEME_KEY).ok()?.filter(|v| !v.is_empty())
}

pub fn apply_theme(theme: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Some(html) = doc.document_element() {
                let _ = html.set_attribute("data-theme", theme);
            }
        }
    }
}
