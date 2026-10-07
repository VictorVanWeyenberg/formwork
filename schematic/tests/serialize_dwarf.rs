use schematic::dnd_model::dwarf_schematic;

#[test]
fn serialize_dwarf_to_json() {
    let dwarf = dwarf_schematic();
    let json = serde_json::to_string_pretty(&dwarf).unwrap();
    std::fs::write("dwarf-current.json", json).unwrap();
}
