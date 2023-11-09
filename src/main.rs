use std::{env, time::Duration};

use ethers::providers::{Http, Provider};
use sqlx::postgres::PgPoolOptions;
use verifying_email_binder::{
    server::handler::serve_http,
    service::{email::send_mails, Context, HttpRpcHandler},
};

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await
        .expect("could not connect to database");

    let provider = Provider::<Http>::try_from(env::var("RPC_URL").expect("RPC_URL must be set"))
        .expect("instance provider error");

    let context = Context {
        db,
        provider,
        guardian_address: env::var("GUARDIAN_ADDRESS").expect("GUARDIAN_ADDRESS must be set"),
        signer: env::var("SIGNER").expect("SIGNER must be set"),
    };

    sqlx::migrate!()
        .run(&context.db)
        .await
        .expect("migrate database error");

    tokio::spawn(async move {
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
        let smtp_user = env::var("SMTP_USER").expect("SMTP_USER must be set");
        let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let db = PgPoolOptions::new()
            .max_connections(50)
            .connect(&database_url)
            .await
            .expect("could not connect to database");
        loop {
            send_mails(&db, &smtp_password, &smtp_user, &smtp_host).await;
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });

    let http = HttpRpcHandler::new(context);
    serve_http("0.0.0.0:3000".parse().unwrap(), http)
        .await
        .unwrap();
}
