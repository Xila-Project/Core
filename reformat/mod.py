import os
import re
import shutil

def to_snake_case(name):
    """Convert PascalCase/CamelCase to snake_case"""
    # Insert underscore before uppercase letters that follow lowercase letters
    s1 = re.sub('([a-z0-9])([A-Z])', r'\1_\2', name)
    # Insert underscore before uppercase letters that are followed by lowercase letters
    s2 = re.sub('([A-Z])([A-Z][a-z])', r'\1_\2', s1)
    return s2.lower()

def rename_rs_files(directory):
    """Rename all .rs files in directory to snake_case"""
    renamed_files = {}
    
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.endswith('.rs'):
                old_path = os.path.join(root, file)
                filename_without_ext = file[:-3]  # Remove .rs extension
                new_filename = to_snake_case(filename_without_ext) + '.rs'
                new_path = os.path.join(root, new_filename)
                
                if old_path != new_path:
                    print(f"Renaming: {old_path} -> {new_path}")
                    shutil.move(old_path, new_path)
                    renamed_files[filename_without_ext] = to_snake_case(filename_without_ext)
    
    return renamed_files

def update_mod_declarations(directory, renamed_files):
    """Update mod declarations in all .rs files"""
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.endswith('.rs'):
                file_path = os.path.join(root, file)
                
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                modified = False
                
                # Pattern to match mod declarations
                mod_pattern = r'mod\s+([A-Z][a-zA-Z0-9_]*)\s*;'
                
                def replace_mod(match):
                    nonlocal modified
                    module_name = match.group(1)
                    snake_case_name = to_snake_case(module_name)
                    if module_name != snake_case_name:
                        modified = True
                        return f'mod {snake_case_name};'
                    return match.group(0)
                
                new_content = re.sub(mod_pattern, replace_mod, content)
                
                # Also handle use statements with module names
                use_pattern = r'use\s+([A-Z][a-zA-Z0-9_]*)::'
                
                def replace_use(match):
                    nonlocal modified
                    module_name = match.group(1)
                    snake_case_name = to_snake_case(module_name)
                    if module_name != snake_case_name:
                        modified = True
                        return f'use {snake_case_name}::'
                    return match.group(0)
                
                new_content = re.sub(use_pattern, replace_use, new_content)
                
                if modified:
                    print(f"Updating mod declarations in: {file_path}")
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(new_content)

def main():
    # Get the current directory or specify the path to your Rust project
    project_directory = input("Enter the path to your Rust project (or press Enter for current directory): ").strip()
    
    if not project_directory:
        project_directory = os.getcwd()
    
    if not os.path.exists(project_directory):
        print(f"Directory {project_directory} does not exist!")
        return
    
    print(f"Processing directory: {project_directory}")
    
    # First, rename all .rs files to snake_case
    print("\n--- Renaming .rs files ---")
    renamed_files = rename_rs_files(project_directory)
    
    # Then update mod declarations in all files
    print("\n--- Updating mod declarations ---")
    update_mod_declarations(project_directory, renamed_files)
    
    print("\n--- Done ---")
    print(f"Renamed {len(renamed_files)} files")
    
    if renamed_files:
        print("Files renamed:")
        for old_name, new_name in renamed_files.items():
            print(f"  {old_name}.rs -> {new_name}.rs")

if __name__ == "__main__":
    main()