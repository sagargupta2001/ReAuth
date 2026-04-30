import re

with open('src/application/harbor/service.rs', 'r') as f:
    content = f.read()

# Fix HarborService struct
struct_match = re.search(r'pub struct HarborService \{([^}]+)\}', content)
if struct_match:
    struct_body = struct_match.group(1)
    new_body = re.sub(r'\s+pub\(crate\)\s+', ' ', struct_body) # Remove existing
    new_body = re.sub(r'([a-zA-Z0-9_]+):', r'pub(crate) \1:', new_body) # Add to all
    content = content[:struct_match.start(1)] + new_body + content[struct_match.end(1):]

# Fix ImportProgress struct
import_match = re.search(r'pub\(crate\) struct ImportProgress \{([^}]+)\}', content)
if import_match:
    import_body = import_match.group(1)
    new_body = re.sub(r'\s+pub\(crate\)\s+', ' ', import_body)
    new_body = re.sub(r'([a-zA-Z0-9_]+):', r'pub(crate) \1:', new_body)
    content = content[:import_match.start(1)] + new_body + content[import_match.end(1):]

with open('src/application/harbor/service.rs', 'w') as f:
    f.write(content)
