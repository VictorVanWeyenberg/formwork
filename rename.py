import os
import re

directories = ['schematic/src', 'schematic/tests', 'schematic-macro/src', 'schematic-macro/tests']

replacements = [
    ("blueprint_trait", "schematic_trait"),
    ("BlueprintError", "SchematicError"),
    ("BlueprintStruct", "SchematicStruct"),
    ("BlueprintField", "SchematicField"),
    ("Blueprint", "Schematic"),
    ("blueprint::", "schematic::"),
    ("blueprint", "schematic"),
    ("Draft", "Unresolved"),
    ("Final", "Resolved"),
    ("draft", "unresolved"),
    ("final_", "resolved_"),
    ("is_vec", "is_collection"),
    ("is_choice", "is_pick"),
    ("is_nested", "is_cascade"),
    ("#[slot(vec", "#[slot(collection"),
    ("#[slot(choice", "#[slot(pick"),
    ("#[slot(nested", "#[slot(cascade"),
    ('meta.path.is_ident("vec")', 'meta.path.is_ident("collection")'),
    ('meta.path.is_ident("choice")', 'meta.path.is_ident("pick")'),
    ('meta.path.is_ident("nested")', 'meta.path.is_ident("cascade")'),
]

for d in directories:
    for root, dirs, files in os.walk(d):
        for f in files:
            if f.endswith('.rs'):
                path = os.path.join(root, f)
                with open(path, 'r') as file:
                    content = file.read()
                
                for old, new in replacements:
                    content = content.replace(old, new)
                
                with open(path, 'w') as file:
                    file.write(content)
print("Done")
