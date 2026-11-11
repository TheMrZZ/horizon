//! Catalog-dependent saved stack projection at its existing caller.
use super::*;
impl Menu {
    /// `CraftingMenu.slotChangedCraftingGrid`.
    pub(super) fn recompute(&mut self, env: &Env) {
        if self.grid_size == 0 {
            return;
        }
        let result = env
            .recipes
            .find_with_items(
                &self.containers[GRID],
                self.grid_size,
                &|id| {
                    env.maps
                        .and_then(|maps| maps.state(id))
                        .map(|(_, scale)| scale)
                },
                env.items,
            )
            .and_then(|(id, stack)| {
                if env.limited_crafting {
                    // `RecipeCraftingHolder.setRecipeUsed`: special recipes or known recipes craft.
                    let is_special = env.recipes.recipe(id).is_some_and(|r| r.is_special());
                    let known = env.known_recipes.is_some_and(|book| book.knows(id));
                    if !is_special && !known {
                        return None;
                    }
                }
                Some(stack)
            });
        self.containers[RESULT][0] = result;
    }
}
