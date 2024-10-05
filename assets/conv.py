import os
import re

def transform_ron_file(input_file, output_file):
    with open(input_file, 'r') as f:
        content = f.read()

    # Regular expression to match the node structure
    pattern = r'\(\s*name:\s*"([^"]+)",\s*ty:\s*"([^"]+)",\s*inner:\s*(\([^)]+\))'

    def replace_node(match):
        name = match.group(1)
        ty = match.group(2)
        inner = match.group(3)
        return f'(\n            name: "{name}",\n            inner: {{\n                "{ty}": {inner}\n            }}'

    # Apply the transformation
    transformed_content = re.sub(pattern, replace_node, content)

    with open(output_file, 'w') as f:
        f.write(transformed_content)

def process_directory(input_dir, output_dir):
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    for filename in os.listdir(input_dir):
        if filename.endswith('.ron'):
            input_path = os.path.join(input_dir, filename)
            output_path = os.path.join(output_dir, filename)
            transform_ron_file(input_path, output_path)
            print(f"Transformed {filename}")

if __name__ == "__main__":
    input_directory = "animation_graphs_old"
    output_directory = "animation_graphs"

    process_directory(input_directory, output_directory)
    print("Transformation complete.")
