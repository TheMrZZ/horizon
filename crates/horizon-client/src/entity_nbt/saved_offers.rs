//! ItemCost predicates remain exact; only the actual sell stack uses its prototype.
use super::*;
/// `AbstractVillager` "Offers" (`MerchantOffers.CODEC`).
pub(super) fn put_offers(map: &mut Compound, m: &Mob, saved: SavedStacks) {
    let Some(t) = m.extra.trade.as_ref().filter(|t| !t.unset) else {
        return;
    };
    map.insert("Offers".into(), offers_to_nbt_saved(&t.offers, saved));
}
/// `ItemCost.CODEC`: id, count (always written) and the exact components.
fn item_cost_to_nbt(stack: &crate::items::ItemStack) -> Tag {
    let mut tag = item_to_nbt(stack);
    if let Tag::Compound(c) = &mut tag
        && !c.contains_key(&"count".into())
    {
        c.insert("count".into(), Tag::Int(stack.count as i32));
    }
    tag
}
/// `MerchantOffers.CODEC`: `{Recipes: [...]}`, each offer without the
/// fields at their defaults.
pub fn offers_to_nbt(offers: &[crate::merchant::MerchantOffer]) -> Tag {
    offers_to_nbt_saved(offers, SavedStacks(None))
}

pub fn offers_to_nbt_with_items(
    offers: &[crate::merchant::MerchantOffer],
    items: &crate::items::Items,
) -> Tag {
    offers_to_nbt_saved(offers, SavedStacks(Some(items)))
}

fn offers_to_nbt_saved(offers: &[crate::merchant::MerchantOffer], saved: SavedStacks) -> Tag {
    let recipes = offers
        .iter()
        .map(|o| {
            let mut entries = vec![
                ("buy", item_cost_to_nbt(&o.base_cost_a.stack)),
                ("sell", saved.stack(&o.result)),
            ];
            // `MerchantOffer.CODEC` omits fields at their defaults.
            if o.uses != 0 {
                entries.push(("uses", Tag::Int(o.uses)));
            }
            if o.max_uses != 4 {
                entries.push(("maxUses", Tag::Int(o.max_uses)));
            }
            if o.price_multiplier != 0.0 {
                entries.push(("priceMultiplier", Tag::Float(o.price_multiplier.to_bits())));
            }
            if o.xp != 1 {
                entries.push(("xp", Tag::Int(o.xp)));
            }
            if !o.reward_exp {
                entries.push(("rewardExp", Tag::Byte(0)));
            }
            if o.special_price_diff != 0 {
                entries.push(("specialPrice", Tag::Int(o.special_price_diff)));
            }
            if o.demand != 0 {
                entries.push(("demand", Tag::Int(o.demand)));
            }
            if let Some(b) = &o.cost_b {
                entries.push(("buyB", item_cost_to_nbt(&b.stack)));
            }
            Tag::compound(entries)
        })
        .collect::<Vec<_>>();
    // An empty list is the field's default: left out.
    if recipes.is_empty() {
        return Tag::Compound(Compound::new());
    }
    Tag::compound([("Recipes", Tag::List(recipes))])
}
