#!/usr/bin/env python3
"""
Fix color conversion issues in the RustIRC GUI components.
Converts .scheme.*.into() patterns to iced::Color::from(.scheme.*) patterns.
"""

import os
import re
import glob

def fix_color_conversions(file_path):
    """Fix color conversion patterns in a single file."""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    # Pattern 1: self.theme.scheme.COLOR.into() -> iced::Color::from(self.theme.scheme.COLOR)
    pattern1 = r'self\.theme\.scheme\.([a-zA-Z_]+)\.into\(\)'
    replacement1 = r'iced::Color::from(self.theme.scheme.\1)'
    content = re.sub(pattern1, replacement1, content)
    
    # Pattern 2: theme.scheme.COLOR.into() -> iced::Color::from(theme.scheme.COLOR)
    pattern2 = r'theme\.scheme\.([a-zA-Z_]+)\.into\(\)'
    replacement2 = r'iced::Color::from(theme.scheme.\1)'
    content = re.sub(pattern2, replacement2, content)
    
    # Pattern 3: self.theme.scheme.COLOR.scale_alpha(N).into() -> iced::Color::from(self.theme.scheme.COLOR.scale_alpha(N))
    pattern3 = r'self\.theme\.scheme\.([a-zA-Z_]+)\.scale_alpha\(([0-9.]+)\)\.into\(\)'
    replacement3 = r'iced::Color::from(self.theme.scheme.\1.scale_alpha(\2))'
    content = re.sub(pattern3, replacement3, content)
    
    # Pattern 4: theme.scheme.COLOR.scale_alpha(N).into() -> iced::Color::from(theme.scheme.COLOR.scale_alpha(N))
    pattern4 = r'theme\.scheme\.([a-zA-Z_]+)\.scale_alpha\(([0-9.]+)\)\.into\(\)'
    replacement4 = r'iced::Color::from(theme.scheme.\1.scale_alpha(\2))'
    content = re.sub(pattern4, replacement4, content)
    
    # Pattern 5: variable.into() where variable is a color -> iced::Color::from(variable)
    # This is more complex - we'll handle specific known color variables
    color_vars = ['background_color', 'border_color', 'text_color', 'surface_color', 'counter_color']
    for var in color_vars:
        pattern = rf'{var}\.into\(\)'
        replacement = f'iced::Color::from({var})'
        content = re.sub(pattern, replacement, content)
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Fixed color conversions in {file_path}")
        return True
    
    return False

def main():
    # Find all Rust files in the components directory
    component_files = glob.glob('/var/home/parobek/Code/RustIRC/crates/rustirc-gui/src/components/**/*.rs', recursive=True)
    
    files_modified = 0
    for file_path in component_files:
        if fix_color_conversions(file_path):
            files_modified += 1
    
    print(f"Modified {files_modified} files")

if __name__ == '__main__':
    main()