use dioxus::prelude::*;

use crate::api;
use crate::settings::{DEFAULT_BAD_MS, DEFAULT_WARN_MS, HealthThresholds};

#[derive(Default)]
struct SettingsState {
    warn_ms: String,
    bad_ms: String,
    saving: bool,
    message: Option<String>,
    error: Option<String>,
}

#[derive(Default)]
struct McpPasswordState {
    /// Loaded from the server: true if a password is currently set.
    password_set: bool,
    /// Set once after first GET, so we don't re-fetch on every render.
    loaded: bool,
    /// New password being typed; never echoed back to the server unless
    /// the user clicks Save.
    new_password: String,
    show: bool,
    saving: bool,
    message: Option<String>,
    error: Option<String>,
}

#[component]
pub fn Settings() -> Element {
    let mut thresholds = use_context::<Signal<HealthThresholds>>();
    let mut cs = use_signal(SettingsState::default);
    let mut mcp = use_signal(McpPasswordState::default);

    // Lazy-load whether the MCP write password is configured. We only
    // care about the boolean — the server never reveals the value.
    {
        let loaded = mcp.read().loaded;
        if !loaded {
            mcp.write().loaded = true;
            spawn(async move {
                if let Ok(s) = api::get_ui_settings().await {
                    mcp.write().password_set = s.mcp_write_password_set;
                }
            });
        }
    }

    // Sync local form fields whenever the context value changes (e.g. on initial load).
    let t = *thresholds.read();
    {
        let mut w = cs.write();
        if w.warn_ms.is_empty() {
            w.warn_ms = t.warn_ms.to_string();
        }
        if w.bad_ms.is_empty() {
            w.bad_ms = t.bad_ms.to_string();
        }
    }

    let cs_ra = cs.read();
    let warn_str = cs_ra.warn_ms.clone();
    let bad_str = cs_ra.bad_ms.clone();
    let saving = cs_ra.saving;
    let message = cs_ra.message.clone();
    let error = cs_ra.error.clone();
    drop(cs_ra);

    let save = move |_| {
        let warn_parsed = cs.read().warn_ms.parse::<u32>();
        let bad_parsed = cs.read().bad_ms.parse::<u32>();
        let (Ok(warn_ms), Ok(bad_ms)) = (warn_parsed, bad_parsed) else {
            let mut w = cs.write();
            w.error = Some("Both fields must be positive whole numbers (ms).".to_string());
            w.message = None;
            return;
        };
        if warn_ms >= bad_ms {
            let mut w = cs.write();
            w.error = Some("Green→Yellow threshold must be smaller than Yellow→Red.".to_string());
            w.message = None;
            return;
        }
        {
            let mut w = cs.write();
            w.saving = true;
            w.error = None;
            w.message = None;
        }
        let new_t = HealthThresholds { warn_ms, bad_ms };
        spawn(async move {
            match api::set_health_thresholds(new_t).await {
                Ok(()) => {
                    thresholds.set(new_t);
                    let mut w = cs.write();
                    w.saving = false;
                    w.message = Some("Saved.".to_string());
                }
                Err(err) => {
                    let mut w = cs.write();
                    w.saving = false;
                    w.error = Some(format!("Save failed: {}", err));
                }
            }
        });
    };

    let reset = move |_| {
        let mut w = cs.write();
        w.warn_ms = DEFAULT_WARN_MS.to_string();
        w.bad_ms = DEFAULT_BAD_MS.to_string();
        w.error = None;
        w.message = None;
    };

    let footer = if let Some(m) = message.clone() {
        rsx! {
            div { style: "color: var(--ok); font-size: 12px;", "{m}" }
        }
    } else if let Some(e) = error.clone() {
        rsx! {
            div { style: "color: var(--danger); font-size: 12px;", "{e}" }
        }
    } else {
        rsx! {}
    };

    // ----- MCP write password card state & handlers -----
    let mcp_ra = mcp.read();
    let mcp_password_set = mcp_ra.password_set;
    let mcp_show = mcp_ra.show;
    let mcp_new = mcp_ra.new_password.clone();
    let mcp_saving = mcp_ra.saving;
    let mcp_message = mcp_ra.message.clone();
    let mcp_error = mcp_ra.error.clone();
    drop(mcp_ra);

    let mcp_save = move |_| {
        let value = mcp.read().new_password.clone();
        if value.trim().is_empty() {
            let mut w = mcp.write();
            w.error = Some("Enter a non-empty password.".to_string());
            w.message = None;
            return;
        }
        {
            let mut w = mcp.write();
            w.saving = true;
            w.error = None;
            w.message = None;
        }
        spawn(async move {
            match api::set_mcp_write_password(&value).await {
                Ok(()) => {
                    let mut w = mcp.write();
                    w.saving = false;
                    w.password_set = true;
                    w.new_password = String::new();
                    w.message = Some("Password saved.".to_string());
                }
                Err(err) => {
                    let mut w = mcp.write();
                    w.saving = false;
                    w.error = Some(format!("Save failed: {}", err));
                }
            }
        });
    };

    let mcp_clear = move |_| {
        {
            let mut w = mcp.write();
            w.saving = true;
            w.error = None;
            w.message = None;
        }
        spawn(async move {
            match api::set_mcp_write_password("").await {
                Ok(()) => {
                    let mut w = mcp.write();
                    w.saving = false;
                    w.password_set = false;
                    w.new_password = String::new();
                    w.message = Some("Password cleared.".to_string());
                }
                Err(err) => {
                    let mut w = mcp.write();
                    w.saving = false;
                    w.error = Some(format!("Clear failed: {}", err));
                }
            }
        });
    };

    let mcp_footer = if let Some(m) = mcp_message.clone() {
        rsx! { div { style: "color: var(--ok); font-size: 12px;", "{m}" } }
    } else if let Some(e) = mcp_error.clone() {
        rsx! { div { style: "color: var(--danger); font-size: 12px;", "{e}" } }
    } else {
        rsx! {}
    };

    let mcp_status_label = if mcp_password_set {
        "currently set"
    } else {
        "not configured"
    };
    let mcp_status_color = if mcp_password_set {
        "var(--ok)"
    } else {
        "var(--text-muted)"
    };
    let mcp_input_type = if mcp_show { "text" } else { "password" };
    let mcp_show_label = if mcp_show { "Hide" } else { "Show" };

    rsx! {
        section { class: "page page--padded",
            div { style: "max-width: 640px; display: flex; flex-direction: column; gap: 14px;",
                div { class: "card",
                    div { class: "card__header",
                        span { class: "card__title", "Reader health thresholds" }
                        span { class: "card__subtitle", "milliseconds since last incoming" }
                    }
                    div { class: "card__body", style: "display: flex; flex-direction: column; gap: 14px;",
                        p { style: "margin: 0; color: var(--text-muted); font-size: 12.5px;",
                            "Below "
                            b { style: "color: var(--ok); font-family: var(--font-mono);", "Green" }
                            " — healthy. Between Green and Yellow — slow. Above "
                            b { style: "color: var(--danger); font-family: var(--font-mono);", "Yellow" }
                            " — stalled. Values are stored on the server in "
                            code { style: "font-family: var(--font-mono);", "ui-settings.json" }
                            " next to the data files."
                        }

                        div { class: "settings-row",
                            label { class: "settings-row__label",
                                span { class: "state state--ok", span { class: "state__dot" } }
                                "Green → Yellow"
                            }
                            div { class: "settings-row__field",
                                input {
                                    class: "filter-input",
                                    r#type: "number",
                                    min: "0",
                                    value: "{warn_str}",
                                    oninput: move |evt| {
                                        let mut w = cs.write();
                                        w.warn_ms = evt.value();
                                        w.message = None;
                                    },
                                }
                                span { class: "settings-row__unit", "ms" }
                            }
                        }

                        div { class: "settings-row",
                            label { class: "settings-row__label",
                                span { class: "state state--bad", span { class: "state__dot" } }
                                "Yellow → Red"
                            }
                            div { class: "settings-row__field",
                                input {
                                    class: "filter-input",
                                    r#type: "number",
                                    min: "0",
                                    value: "{bad_str}",
                                    oninput: move |evt| {
                                        let mut w = cs.write();
                                        w.bad_ms = evt.value();
                                        w.message = None;
                                    },
                                }
                                span { class: "settings-row__unit", "ms" }
                            }
                        }

                        {footer}
                    }
                    div { class: "card__footer", style: "display: flex; justify-content: flex-end; gap: 6px; padding: 10px 14px;",
                        button { class: "btn btn--ghost btn--sm", onclick: reset, "Reset to defaults" }
                        button {
                            class: "btn btn--primary btn--sm",
                            disabled: saving,
                            onclick: save,
                            if saving { "Saving…" } else { "Save" }
                        }
                    }
                }

                // ----- MCP write password card -----
                div { class: "card",
                    div { class: "card__header",
                        span { class: "card__title", "MCP write password" }
                        span {
                            class: "card__subtitle",
                            style: "color: {mcp_status_color};",
                            "{mcp_status_label}"
                        }
                    }
                    div { class: "card__body", style: "display: flex; flex-direction: column; gap: 14px;",
                        p { style: "margin: 0; color: var(--text-muted); font-size: 12.5px;",
                            "Gates the MCP write tools "
                            code { style: "font-family: var(--font-mono);", "delete_row" }
                            " and "
                            code { style: "font-family: var(--font-mono);", "insert_or_replace_row" }
                            ". Stored as a salted SHA-256 hash on the server. AI clients "
                            "must request the value via MCP elicitation and NEVER cache it — "
                            "see the "
                            code { style: "font-family: var(--font-mono);", "mcp_write_password_policy" }
                            " prompt exposed by the server."
                        }

                        div { class: "settings-row",
                            label { class: "settings-row__label",
                                if mcp_password_set {
                                    "Replace password"
                                } else {
                                    "Set password"
                                }
                            }
                            div { class: "settings-row__field",
                                input {
                                    class: "filter-input",
                                    r#type: "{mcp_input_type}",
                                    autocomplete: "new-password",
                                    placeholder: "New password",
                                    value: "{mcp_new}",
                                    oninput: move |evt| {
                                        let mut w = mcp.write();
                                        w.new_password = evt.value();
                                        w.message = None;
                                    },
                                }
                                button {
                                    class: "btn btn--ghost btn--sm",
                                    onclick: move |_| {
                                        let mut w = mcp.write();
                                        w.show = !w.show;
                                    },
                                    "{mcp_show_label}"
                                }
                            }
                        }

                        {mcp_footer}
                    }
                    div { class: "card__footer", style: "display: flex; justify-content: flex-end; gap: 6px; padding: 10px 14px;",
                        if mcp_password_set {
                            button {
                                class: "btn btn--ghost btn--sm",
                                disabled: mcp_saving,
                                onclick: mcp_clear,
                                "Clear"
                            }
                        }
                        button {
                            class: "btn btn--primary btn--sm",
                            disabled: mcp_saving,
                            onclick: mcp_save,
                            if mcp_saving { "Saving…" } else { "Save" }
                        }
                    }
                }
            }
        }
    }
}
