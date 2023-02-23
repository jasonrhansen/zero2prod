use axum::BoxError;
use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    run()?.await?;

    Ok(())
}
