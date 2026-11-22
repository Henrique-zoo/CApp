//! Ferramenta do operador para a preparação privada do eleitorado.

use backend::modules::elections::infrastructure::PgVoterRegistry;
use sqlx::postgres::PgPoolOptions;
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [election, course] = args.as_slice() else {
        return Err(
            "uso: configure_election_course <UUID da eleição> <curso institucional>".into(),
        );
    };
    let election = Uuid::parse_str(election)?;
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    PgVoterRegistry::configure_course(&pool, election, course).await?;
    pool.close().await;
    println!("Curso configurado para a eleição {election}.");
    Ok(())
}
