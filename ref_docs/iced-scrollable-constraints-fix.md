# Iced 0.13 Scrollable Widget Constraints Fix

**Date**: 2025-08-26 10:30 PM EDT  
**Issue**: Material Demo panic with scrollable widget  
**Resolution**: Successfully fixed by constraining content height

## Problem

When attempting to run the Material Design 3 demo with `cargo run -- --material-demo`, the application would panic with:

```
thread 'main' panicked at iced_widget-0.13.4/src/scrollable.rs:118:9:
scrollable content must not fill its vertical scrolling axis
```

## Root Cause

In Iced 0.13, the scrollable widget enforces a constraint that its content cannot use `Length::Fill` in the scrolling direction (vertical for vertical scrolling, horizontal for horizontal scrolling). This is because the scrollable needs to know the actual size of its content to determine scrollbar position and size.

The issue was that components inside the scrollable (particularly `MaterialListItem`) were using `center_y(Length::Fill)` which attempted to fill the vertical axis.

## Solution Applied

The fix involved wrapping the scrollable content in a container with explicit height constraints:

```rust
// Before (causes panic):
let scroll_content = column![/* sections */]
    .width(Length::Fill);  // This could propagate Fill vertically

// After (fixed):
let scroll_content = container(
    column![/* sections */]
        .padding(20)
        .spacing(10)
)
.width(Length::Fill)      // Width can be Fill (horizontal axis)
.height(Length::Shrink);  // Explicitly set height to Shrink
```

Key changes:
1. Wrapped the content column in a container
2. Explicitly set `height(Length::Shrink)` on the container
3. This ensures content sizes itself based on children, not available space

## Research Process

1. Used Context7 MCP tool to get Iced 0.13.1 documentation
2. Searched Brave for specific error message and found GitHub issue #2863
3. Discovered that scrollable content must have constrained dimensions
4. Applied the recommended solution pattern

## Technical Details

### Iced Scrollable Constraints
- Content inside scrollable must not use `Fill` in scroll direction
- Use `Shrink` or fixed sizes for content dimensions
- The scrollable widget itself can use `Fill` for its dimensions
- Nested components must also respect these constraints

### Material Components Affected
Components that may need adjustment when used in scrollables:
- `MaterialListItem` - uses `center_y(Length::Fill)` internally
- Any component using container with Fill in vertical axis
- Layout helpers that propagate Fill dimensions

## Verification

After applying the fix:
- Demo launches successfully without panic
- All Material Design 3 components display properly
- Scrolling functionality works as expected
- No runtime errors or warnings

## Lessons Learned

1. **Always constrain scrollable content**: Never use `Fill` in the scroll direction
2. **Explicit dimensions preferred**: Use explicit `Shrink` or fixed sizes
3. **Container wrapping**: Wrapping in a container with explicit dimensions is a clean solution
4. **Component awareness**: Be aware of how nested components use dimensions

## References

- [Iced GitHub Issue #2863](https://github.com/iced-rs/iced/issues/2863)
- [Iced 0.13 Scrollable Documentation](https://docs.rs/iced/0.13.1/iced/widget/scrollable/index.html)
- Context7 Library ID: /iced-rs/iced/0_13_1