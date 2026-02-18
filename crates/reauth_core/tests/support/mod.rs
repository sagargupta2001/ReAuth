use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
use axum::Router;
use reauth_core::adapters::web::router::create_router;
use reauth_core::initialize_for_tests;
use reauth_core::AppState;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tower::ServiceExt;

pub struct TestContext {
    pub app_state: AppState,
    pub router: Router,
    _temp_dir: TempDir,
    _env_guard: EnvGuard,
}

impl TestContext {
    pub async fn new() -> Self {
        Self::new_with_seed(false).await
    }

    #[allow(dead_code)]
    pub async fn new_with_seed(seed: bool) -> Self {
        Self::new_internal(seed).await
    }

    async fn new_internal(seed: bool) -> Self {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("reauth-test.db");
        let db_url = format!("sqlite:{}", db_path.to_string_lossy());

        let mut env_vars = vec![
            ("REAUTH__DATABASE__URL", db_url),
            (
                "REAUTH__DATABASE__DATA_DIR",
                temp_dir.path().to_string_lossy().to_string(),
            ),
            ("REAUTH__LOGGING__LEVEL", "warn".to_string()),
        ];

        if !seed {
            env_vars.push(("REAUTH_TEST_SKIP_SEED", "1".to_string()));
        }

        let env_guard = EnvGuard::set_all(env_vars);

        let app_state = initialize_for_tests()
            .await
            .expect("failed to initialize app state");

        let router = create_router(app_state.clone(), app_state.plugins_path.clone());

        Self {
            app_state,
            router,
            _temp_dir: temp_dir,
            _env_guard: env_guard,
        }
    }

    pub async fn request(&self, request: Request<Body>) -> Response<Body> {
        self.router
            .clone()
            .oneshot(request)
            .await
            .expect("request failed")
    }

    #[allow(dead_code)]
    pub fn plugins_path(&self) -> PathBuf {
        self.app_state.plugins_path.clone()
    }
}

struct EnvGuard {
    previous: HashMap<&'static str, Option<String>>,
}

impl EnvGuard {
    fn set_all(vars: Vec<(&'static str, String)>) -> Self {
        let mut previous = HashMap::new();
        for (key, value) in vars {
            let existing = std::env::var(key).ok();
            previous.insert(key, existing);
            std::env::set_var(key, value);
        }
        Self { previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.previous.drain() {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}
