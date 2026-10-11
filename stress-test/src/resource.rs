use crate::inventory::item::Item;

pub enum Resource {
    Action,
    BonusAction,
    Reaction,
    Custom(String),
    SpellSlot { minimum_level: usize },
    Item(Item),
}
