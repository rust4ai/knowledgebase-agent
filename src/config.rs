use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub s3_region: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_bucket: String,
    pub s3_endpoint: Option<String>,
    pub openai_api_key: String,
    pub openai_model: String,
    pub host: String,
    pub port: u16,
    pub admin_password: String,
    pub knowledgebase_name: String,
    pub system_prompt_extra: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL required"),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "nyc3".into()),
            s3_access_key: env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY required"),
            s3_secret_key: env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY required"),
            s3_bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "knowledgebase-docs".into()),
            s3_endpoint: env::var("S3_ENDPOINT").ok().filter(|s| !s.is_empty()),
            openai_api_key: env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY required"),
            openai_model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5.4".into()),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("PORT must be a number"),
            admin_password: env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD required"),
            knowledgebase_name: env::var("KNOWLEDGEBASE_NAME")
                .unwrap_or_else(|_| "Knowledgebase".into()),
            system_prompt_extra: env::var("SYSTEM_PROMPT_EXTRA")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }
}
