# Dioxus v0.6 Components and Hooks Reference

This document provides a comprehensive reference for Dioxus v0.6 components, hooks, and desktop application development patterns.

## Table of Contents
- [Core Concepts](#core-concepts)
- [Component Definition](#component-definition)
- [Props and Attributes](#props-and-attributes)
- [State Management Hooks](#state-management-hooks)
- [Effect and Lifecycle Hooks](#effect-and-lifecycle-hooks)
- [Resource and Async Hooks](#resource-and-async-hooks)
- [Context API](#context-api)
- [Custom Hooks](#custom-hooks)
- [Desktop Platform](#desktop-platform)
- [Rules of Hooks](#rules-of-hooks)

## Core Concepts

Dioxus uses a React-like component model with hooks for state management and side effects.

### Essential Imports

```rust
use dioxus::prelude::*;
```

This imports all essential Dioxus types including:
- `Element` - Represents a UI node
- `rsx!` - Macro for writing UI descriptions
- Component macros and hooks
- Props traits and attributes

## Component Definition

### Basic Component

```rust
#[component]
fn App() -> Element {
    rsx! {
        div { "Hello, world!" }
    }
}
```

### Component with Props

```rust
#[component]
fn Header(title: String, color: String) -> Element {
    rsx! {
        div {
            background_color: "{color}",
            h1 { "{title}" }
        }
    }
}
```

### Component with Optional Props

```rust
#[component]
fn Button(
    text: Option<String>,  // Automatically optional, defaults to None
) -> Element {
    rsx! {
        button { "{text.unwrap_or("button".to_string())}" }
    }
}
```

### Component with Default Props

```rust
#[component]
fn Button(
    #[props(default)]
    text: String,  // Defaults to empty string
    #[props(default = "red".to_string())]
    color: String,  // Defaults to "red"
) -> Element {
    rsx! {
        button {
            color: color,
            "{text}"
        }
    }
}
```

## Props and Attributes

### Reactive Props

```rust
#[component]
fn Counter(count: ReadSignal<i32>) -> Element {
    // Reactive props automatically trigger updates
    let doubled_count = use_memo(move || count() * 2);
    rsx! {
        div {
            "Count: {count}"
            "Doubled: {doubled_count}"
        }
    }
}
```

### Making Non-Reactive Props Work with Hooks

```rust
#[component]
fn Counter(count: i32) -> Element {
    // Use use_reactive to add non-reactive props as dependencies
    let doubled_count = use_memo(use_reactive!(|count| count * 2));
    rsx! {
        div {
            "Count: {count}"
            "Doubled: {doubled_count}"
        }
    }
}
```

### Extended Attributes

```rust
#[component]
fn Button(
    #[props(extends = GlobalAttributes, extends = button)]
    attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        button { ..attributes, "Click me" }
    }
}
```

## State Management Hooks

### use_signal

Creates a reactive signal with an initial value:

```rust
fn Counter() -> Element {
    let mut count = use_signal(|| 0);
    
    rsx! {
        div {
            h1 { "Count: {count}" }
            button {
                onclick: move |_| count += 1,
                "Increment"
            }
        }
    }
}
```

### use_memo

Creates a memoized value that recomputes only when dependencies change:

```rust
fn Calculator() -> Element {
    let input = use_signal(|| 5);
    let squared = use_memo(move || input() * input());
    
    rsx! {
        div { "Input: {input}, Squared: {squared}" }
    }
}
```

### Global Signals

```rust
// Create a global signal accessible anywhere
static COUNT: GlobalSignal<i32> = Signal::global(|| 0);

fn Component() -> Element {
    rsx! {
        button {
            onclick: move |_| *COUNT.write() += 1,
            "Global count: {COUNT}"
        }
    }
}
```

## Effect and Lifecycle Hooks

### use_effect

Runs side effects after render and provides cleanup:

```rust
fn Component() -> Element {
    let mut count = use_signal(|| 0);
    
    use_effect(move || {
        // This runs when count changes
        println!("Count changed to: {}", count());
        
        // Optional cleanup (runs on unmount or before next effect)
        move || {
            println!("Cleaning up...");
        }
    });
    
    rsx! {
        button {
            onclick: move |_| count += 1,
            "Count: {count}"
        }
    }
}
```

### DOM Manipulation with use_effect

```rust
fn CanvasComponent() -> Element {
    let mut count = use_signal(|| 0);
    
    use_effect(move || {
        let count = count.read();
        // Manipulate DOM directly
        document::eval(&format!(
            r#"var c = document.getElementById("my-canvas");
            var ctx = c.getContext("2d");
            ctx.fillText("{count}", 10, 50);"#
        ));
    });
    
    rsx! {
        canvas { id: "my-canvas" }
        button {
            onclick: move |_| count += 1,
            "Update Canvas"
        }
    }
}
```

### use_mounted

Check if a component is mounted:

```rust
fn MountedChecker() -> Element {
    let mounted = use_mounted();
    
    rsx! {
        button {
            onclick: move |_| {
                if mounted.is_mounted() {
                    println!("Component is mounted");
                }
            },
            "Check Status"
        }
    }
}
```

## Resource and Async Hooks

### use_resource

Manages async operations with automatic rerun on dependency changes:

```rust
async fn fetch_data(id: u32) -> Result<String, String> {
    // Async fetch logic
    Ok(format!("Data for ID {}", id))
}

fn DataFetcher() -> Element {
    let id = use_signal(|| 1);
    let resource = use_resource(move || fetch_data(id()));
    
    rsx! {
        match &*resource.read_unchecked() {
            Some(Ok(data)) => rsx! { div { "{data}" } },
            Some(Err(e)) => rsx! { div { "Error: {e}" } },
            None => rsx! { div { "Loading..." } }
        }
    }
}
```

### use_future

For running async operations without managing results:

```rust
fn AsyncComponent() -> Element {
    use_future(move || async move {
        // Async operation
        println!("Async operation complete");
    });
    
    rsx! { div { "Running async..." } }
}
```

### use_coroutine

For long-running async operations:

```rust
fn CoroutineComponent() -> Element {
    use_coroutine(|rx| async move {
        loop {
            // Long-running task
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("Tick");
        }
    });
    
    rsx! { div { "Coroutine running..." } }
}
```

## Context API

### Providing Context

```rust
#[component]
fn App() -> Element {
    // Provide a signal to all children
    use_context_provider(|| Signal::new(0));
    
    rsx! {
        Child {}
    }
}
```

### Consuming Context

```rust
#[component]
fn Child() -> Element {
    // Access the signal from parent
    let signal: Signal<i32> = use_context();
    
    rsx! {
        div { "Value from context: {signal}" }
    }
}
```

## Custom Hooks

Create reusable logic by composing existing hooks:

```rust
fn use_counter(initial: i32) -> (Signal<i32>, impl Fn()) {
    let mut count = use_signal(|| initial);
    let increment = move || count += 1;
    (count, increment)
}

fn Counter() -> Element {
    let (count, increment) = use_counter(0);
    
    rsx! {
        button {
            onclick: move |_| increment(),
            "Count: {count}"
        }
    }
}
```

### Window Size Hook Example

```rust
fn use_window_size() -> Signal<(i32, i32)> {
    let mut size = use_signal(|| (800, 600));
    
    use_effect(move || {
        // Set up resize listener
        let handler = window::add_event_listener("resize", move |_| {
            let window = web_sys::window().unwrap();
            let width = window.inner_width().unwrap().as_f64().unwrap() as i32;
            let height = window.inner_height().unwrap().as_f64().unwrap() as i32;
            size.set((width, height));
        });
        
        // Return cleanup
        move || handler.remove()
    });
    
    size
}
```

## Desktop Platform

### Launching a Desktop App

```rust
use dioxus::prelude::*;

fn main() {
    dioxus_desktop::launch(app);
}

fn app() -> Element {
    rsx! {
        div { "Hello Desktop!" }
    }
}
```

### Desktop Configuration

```rust
fn main() {
    dioxus_desktop::Config::new()
        .with_window(
            dioxus_desktop::WindowConfig::default()
                .with_title("My App")
                .with_resizable(true)
                .with_inner_size(800, 600)
        )
        .launch(app);
}
```

### Desktop-Specific Hooks

```rust
fn DesktopApp() -> Element {
    // Access window functions
    let window = use_window();
    
    rsx! {
        button {
            onclick: move |_| {
                if let Some(win) = window {
                    win.set_title("New Title");
                    win.set_minimized(false);
                }
            },
            "Change Window Title"
        }
    }
}
```

## Rules of Hooks

Hooks must follow these rules to work correctly:

### ✅ Valid Hook Usage

```rust
fn Component() -> Element {
    // ✅ At the root of component
    let state = use_signal(|| 0);
    
    // ✅ At the root of custom hook
    fn use_custom() -> Signal<i32> {
        use_signal(|| 0)
    }
    
    todo!()
}
```

### ❌ Invalid Hook Usage

```rust
fn Component() -> Element {
    let condition = use_signal(|| true);
    
    // ❌ Inside conditionals
    if condition() {
        let state = use_signal(|| 0);
    }
    
    // ❌ Inside loops
    for i in 0..5 {
        let state = use_signal(|| i);
    }
    
    // ❌ Inside event handlers
    rsx! {
        button {
            onclick: move |_| {
                let state = use_signal(|| 0);
            },
        }
    }
    
    // ❌ Inside closures
    let value = use_signal(|| {
        let inner = use_signal(|| 0);
        inner()
    });
    
    todo!()
}
```

## Key Features Summary

- **Component Macro**: `#[component]` simplifies prop handling
- **RSX Macro**: `rsx!` provides JSX-like syntax for UI
- **Reactive System**: Signals automatically trigger re-renders
- **Hook-Based**: All state and effects managed through hooks
- **Desktop Support**: Full webview-based desktop applications
- **Type Safety**: Full Rust type safety throughout
- **Performance**: Virtual DOM with efficient diffing

## Common Patterns

### Loading States

```rust
fn DataLoader() -> Element {
    let data = use_resource(|| async { fetch_data().await });
    
    rsx! {
        match &*data.read_unchecked() {
            None => rsx! { div { "Loading..." } },
            Some(Ok(val)) => rsx! { div { "{val}" } },
            Some(Err(e)) => rsx! { div { "Error: {e}" } },
        }
    }
}
```

### Form Handling

```rust
fn Form() -> Element {
    let mut input = use_signal(String::new);
    
    rsx! {
        input {
            value: "{input}",
            oninput: move |evt| input.set(evt.value()),
        }
        button {
            onclick: move |_| {
                println!("Submitted: {}", input());
                input.set(String::new());
            },
            "Submit"
        }
    }
}
```

### List Rendering

```rust
fn TodoList() -> Element {
    let todos = use_signal(|| vec!["Task 1", "Task 2"]);
    
    rsx! {
        ul {
            for (i, todo) in todos().iter().enumerate() {
                li { key: "{i}", "{todo}" }
            }
        }
    }
}
```