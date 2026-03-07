ALTER TABLE harbor_jobs ADD COLUMN artifact_path TEXT;
ALTER TABLE harbor_jobs ADD COLUMN artifact_filename TEXT;
ALTER TABLE harbor_jobs ADD COLUMN artifact_content_type TEXT;
