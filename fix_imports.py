import os
import re

imports = """use crate::application::harbor::schema::*;
use crate::application::harbor::types::*;
use crate::domain::harbor_job::HarborJob;
use crate::domain::harbor_job_conflict::HarborJobConflict;
use crate::domain::flow::models::FlowDraft;
use crate::domain::role::Role;
use crate::domain::user::User;
use crate::domain::pagination::PageRequest;
use crate::ports::transaction_manager::Transaction;
use chrono::Utc;
"""

files = ['src/application/harbor/export.rs', 'src/application/harbor/import.rs', 'src/application/harbor/jobs.rs', 'src/application/harbor/utils.rs']

for file in files:
    with open(file, 'r') as f:
        content = f.read()
    
    # insert imports
    content = imports + content
    
    # For utils, make functions pub(crate)
    if 'utils.rs' in file:
        content = re.sub(r'
fn ', '
pub(crate) fn ', content)
    
    # Replace HARBOR_BUNDLE_VERSION, HARBOR_SCHEMA_VERSION with their super:: equivalents or export them
    content = content.replace('HARBOR_BUNDLE_VERSION', 'super::service::HARBOR_BUNDLE_VERSION')
    content = content.replace('HARBOR_SCHEMA_VERSION', 'super::service::HARBOR_SCHEMA_VERSION')
    content = content.replace('HARBOR_JOB_TYPE_EXPORT', 'super::service::HARBOR_JOB_TYPE_EXPORT')
    content = content.replace('HARBOR_JOB_TYPE_IMPORT', 'super::service::HARBOR_JOB_TYPE_IMPORT')
    content = content.replace('HARBOR_JOB_STATUS_IN_PROGRESS', 'super::service::HARBOR_JOB_STATUS_IN_PROGRESS')
    
    with open(file, 'w') as f:
        f.write(content)

# We also need to expose these constants in service.rs
with open('src/application/harbor/service.rs', 'r') as f:
    service = f.read()

service = service.replace('const HARBOR_BUNDLE_VERSION', 'pub(crate) const HARBOR_BUNDLE_VERSION')
service = service.replace('const HARBOR_SCHEMA_VERSION', 'pub(crate) const HARBOR_SCHEMA_VERSION')
service = service.replace('const HARBOR_JOB_TYPE_EXPORT', 'pub(crate) const HARBOR_JOB_TYPE_EXPORT')
service = service.replace('const HARBOR_JOB_TYPE_IMPORT', 'pub(crate) const HARBOR_JOB_TYPE_IMPORT')
service = service.replace('const HARBOR_JOB_STATUS_IN_PROGRESS', 'pub(crate) const HARBOR_JOB_STATUS_IN_PROGRESS')
service = service.replace('struct ImportProgress', 'pub(crate) struct ImportProgress')

with open('src/application/harbor/service.rs', 'w') as f:
    f.write(service)

print("Imports fixed")
