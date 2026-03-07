//! DCC file transfer progress component

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DccTransferInfo {
    pub filename: String,
    pub size: u64,
    pub transferred: u64,
    pub speed: f64,
    pub direction: DccDirection,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DccDirection {
    Send,
    Receive,
}

#[component]
pub fn DccTransfer(transfer: DccTransferInfo) -> Element {
    let progress = if transfer.size > 0 {
        (transfer.transferred as f64 / transfer.size as f64 * 100.0) as u32
    } else {
        0
    };

    let direction_label = match transfer.direction {
        DccDirection::Send => "Sending",
        DccDirection::Receive => "Receiving",
    };

    let size_display = format_size(transfer.size);
    let transferred_display = format_size(transfer.transferred);
    let speed_display = format_speed(transfer.speed);

    rsx! {
        div {
            class: "flex flex-col gap-1 p-2 bg-[var(--surface-color,#2d2d2d)] rounded border border-[var(--border-color,#333)] text-xs",

            div {
                class: "flex justify-between text-[var(--text-color,#e0e0e0)]",
                span { "{direction_label}: {transfer.filename}" }
                span { "{transferred_display} / {size_display}" }
            }

            // Progress bar
            div {
                class: "w-full h-2 bg-[var(--bg-color,#1a1a1a)] rounded overflow-hidden",
                div {
                    class: "h-full bg-[var(--accent-color,#4ecdc4)] transition-all duration-300",
                    style: "width: {progress}%",
                }
            }

            div {
                class: "flex justify-between text-[var(--text-muted,#888)]",
                span { "{progress}%" }
                span { "{speed_display}" }
            }
        }
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_048_576.0)
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}
