//! Captures of actual ItemStacks, distinct from passive ItemStackTemplates.
use crate::items::{ItemStack, Items};

/// Capture one live stack's saved patch without changing its live model or
/// recursively materializing any templates in its component payloads.
/// The existing catalog-aware encoder remains the prototype authority.
pub fn capture(stack: &ItemStack, items: &Items) -> ItemStack {
    let mut saved = stack.clone();
    if stack
        .data()
        .is_some_and(|data| data.other("minecraft:max_stack_size").is_some())
        && crate::item_data::encode_components_with_items(stack, items)
            .and_then(|patch| patch.get("minecraft:max_stack_size").cloned())
            .is_none()
    {
        saved.data_mut().set_other("minecraft:max_stack_size", None);
        saved.tidy();
    }
    saved
}

/// ContainerHelper and fromStack capture only these actual roots. Nested
/// template patches remain as supplied until a caller materializes them.
pub fn capture_slots(slots: &mut [Option<ItemStack>], items: &Items) {
    for stack in slots.iter_mut().flatten() {
        *stack = capture(stack, items);
    }
}

#[cfg(test)]
mod tests;
