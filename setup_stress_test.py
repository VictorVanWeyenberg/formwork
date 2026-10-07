import os
import shutil
import re

# 1. Update Cargo.toml
cargo_toml_path = 'stress-test/Cargo.toml'
with open(cargo_toml_path, 'a') as f:
    f.write('''
schematic = { path = "../schematic" }
schematic-macro = { path = "../schematic-macro" }
serde = { version = "1", features = ["derive"] }
serde_with = "3.22"
regex = "1.13"
uuid = { version = "1", features = ["v4", "serde"] }
either = { version = "1.17", features = ["serde"] }
''')

# 2. Copy source files
src_dir = os.path.expanduser('~/Desktop/fifth/fifth_domain/src')
dst_dir = 'stress-test/src'

# Remove default src/lib.rs if exists
if os.path.exists(dst_dir):
    shutil.rmtree(dst_dir)

shutil.copytree(src_dir, dst_dir)

# 3. Clean up unwanted modules
blueprint_dir = os.path.join(dst_dir, 'blueprint')
patch_dir = os.path.join(dst_dir, 'patch')
tokenize_dir = os.path.join(dst_dir, 'tokenize')

for d in [blueprint_dir, patch_dir, tokenize_dir]:
    if os.path.exists(d):
        shutil.rmtree(d)

lib_rs_path = os.path.join(dst_dir, 'lib.rs')
with open(lib_rs_path, 'r') as f:
    lib_rs = f.read()

lib_rs = re.sub(r'pub mod blueprint;\n', '', lib_rs)
lib_rs = re.sub(r'pub mod patch;\n', '', lib_rs)
lib_rs = re.sub(r'pub mod tokenize;\n', '', lib_rs)
lib_rs = re.sub(r'#\[cfg\(feature = "tokenize"\)\]\n', '', lib_rs)

with open(lib_rs_path, 'w') as f:
    f.write(lib_rs)

# 4. Strip builder related code
for root, _, files in os.walk(dst_dir):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            with open(filepath, 'r') as f:
                content = f.read()

            # Remove `#[cfg_attr(feature = "builders", ...)]`
            # This handles multi-line attributes too. We can do this with a regex
            # that matches #[cfg_attr(feature = "builders", ... )] up to the matching closing bracket.
            
            # Simple regex for single-line
            content = re.sub(r'#\[cfg_attr\(feature = "builders",.*?\)\]\n', '', content)
            
            # For multi-line #[cfg_attr(feature = "builders", derive(TypedBuilder), builder(mutators( ... )))]
            # We can use a more robust approach:
            while True:
                match = re.search(r'#\[cfg_attr\(feature = "builders",', content)
                if not match:
                    break
                start = match.start()
                open_brackets = 0
                end = start
                for i in range(start, len(content)):
                    if content[i] == '[':
                        open_brackets += 1
                    elif content[i] == ']':
                        open_brackets -= 1
                        if open_brackets == 0:
                            end = i
                            break
                if end > start:
                    # Remove the newline after the attribute if it exists
                    if end + 1 < len(content) and content[end + 1] == '\n':
                        content = content[:start] + content[end + 2:]
                    else:
                        content = content[:start] + content[end + 1:]
                else:
                    break

            # Remove `#[cfg(feature = "builders")]\nuse typed_builder::TypedBuilder;\n`
            content = re.sub(r'#\[cfg\(feature = "builders"\)\]\s*use typed_builder::TypedBuilder;\n', '', content)
            
            # Remove direct `use typed_builder::TypedBuilder;\n`
            content = re.sub(r'use typed_builder::TypedBuilder;\n', '', content)
            
            with open(filepath, 'w') as f:
                f.write(content)

print("Setup completed successfully.")
