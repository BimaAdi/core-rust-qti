use clap::{Args, Parser, Subcommand};
use core_rust_qti::{
    cli::{
        auth,
        db::{db_generate, db_list, db_migrate, db_revert},
    },
    core::db::init_pool,
    settings::get_config,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Database related command
    Db(DbArgs),
    /// Authentication related command
    Auth(AuthArgs),
}

#[derive(Debug, Args)]
struct AuthArgs {
    #[command(subcommand)]
    command: AuthCommands,
}

#[derive(Debug, Subcommand)]
enum AuthCommands {
    /// Create new user
    CreateUser {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
    },
}

#[derive(Debug, Args)]
struct DbArgs {
    #[command(subcommand)]
    command: DbCommands,
}

#[derive(Debug, Subcommand)]
enum DbCommands {
    /// Generate new migration file
    Generate { migration_name: String },
    /// List all migration
    List,
    /// Run all pending migration
    Migrate,
    /// Revert latest migration
    Revert,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Db(db_args) => match &db_args.command {
            DbCommands::Generate { migration_name } => {
                println!("generate migration: {migration_name:?}");
                let _ = dotenvy::dotenv();
                db_generate(migration_name).await;
            }
            DbCommands::List => {
                println!("list migration");
                let _ = dotenvy::dotenv();
                let config = get_config();
                db_list(&config).await;
            }
            DbCommands::Migrate => {
                println!("run all pending migration");
                let _ = dotenvy::dotenv();
                let config = get_config();
                println!("run migration on {}", config.database_url);
                db_migrate(&config).await;
            }
            DbCommands::Revert => {
                println!("revert latest migration");
                let _ = dotenvy::dotenv();
                let config = get_config();
                println!("{}", config.database_url);
                db_revert(&config).await;
            }
        },
        Commands::Auth(auth_args) => match &auth_args.command {
            AuthCommands::CreateUser { username, password } => {
                println!("create user: {username:?}");
                let _ = dotenvy::dotenv();
                let config = get_config();
                let pool = init_pool(&config).await;
                auth::create_user(&pool, username, password).await.unwrap();
            }
        },
    }
}
