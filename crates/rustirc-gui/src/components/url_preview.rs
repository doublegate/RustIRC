//! URL preview component (placeholder for future link previews)

use dioxus::prelude::*;

#[component]
pub fn UrlPreview(url: String) -> Element {
    rsx! {
        div {
            class: "flex items-center gap-2 p-2 my-1 rounded bg-[var(--surface-color,#2d2d2d)] border border-[var(--border-color,#333)] text-xs",
            a {
                class: "text-[var(--link-color,#0088ff)] underline truncate",
                href: "{url}",
                target: "_blank",
                "{url}"
            }
        }
    }
}
