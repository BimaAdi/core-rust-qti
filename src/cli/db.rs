use tokio::process::Command;

use crate::settings::Config;

pub async fn db_generate(migration_name: &String) {
    let _ = Command::new("sqlx")
        .arg("migrate")
        .arg("add")
        .arg(migration_name)
        .arg("-r")
        .status()
        .await
        .unwrap();
}

pub async fn db_list(config: &Config) {
    let _ = Command::new("sqlx")
        .arg("migrate")
        .arg("info")
        .arg("-D")
        .arg(&config.database_url)
        .status()
        .await
        .unwrap();
}

pub async fn db_migrate(config: &Config) {
    let _ = Command::new("sqlx")
        .arg("migrate")
        .arg("run")
        .arg("-D")
        .arg(&config.database_url)
        .status()
        .await
        .unwrap();
}

pub async fn db_revert(config: &Config) {
    let _ = Command::new("sqlx")
        .arg("migrate")
        .arg("revert")
        .arg("-D")
        .arg(&config.database_url)
        .status()
        .await
        .unwrap();
}
