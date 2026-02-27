use leptos::*;

#[component]
pub fn StatusBadge(status: String) -> impl IntoView {
    let (color, text) = match status.as_str() {
        "active" | "completed" => ("green", status),
        "idle" | "pending" => ("yellow", status),
        "error" | "failed" => ("red", status),
        "in_progress" => ("blue", "In Progress"),
        _ => ("gray", status),
    };

    view! {
        <span class=format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-{}-100 text-{}-800", color, color)>
            {text}
        </span>
    }
}
