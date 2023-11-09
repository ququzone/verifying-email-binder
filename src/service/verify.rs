use ethers::signers::{LocalWallet, Signer};

use crate::{
    contracts::guardian::get_hash,
    service::{
        code::BindCode,
        error::{Result, ServiceError},
        Context,
    },
};

pub async fn verify_code(
    context: &Context,
    account: String,
    email: String,
    code: String,
) -> Result<String> {
    let codes = sqlx::query_as::<_, BindCode>(
        "select id, account, email, code, status, created_at, updated_at from bind_code where account = $1 and email = $2 and code = $3 and status = $4 order by id desc limit 1",
    ).bind(&account).bind(&email).bind(&code).bind(1i16).fetch_all(&context.db).await?;

    if codes.is_empty() || codes[0].created_at.timestamp() + 360 < chrono::Local::now().timestamp()
    {
        return Err(ServiceError::InvalidRequest("error code".to_string()));
    }

    let wallet = match context.signer.parse::<LocalWallet>() {
        Ok(w) => w,
        Err(err) => return Err(ServiceError::InvalidRequest(err.to_string())),
    };
    let hash = match get_hash(
        context.provider.clone(),
        &context.guardian_address,
        &account,
        &email,
    )
    .await
    {
        Ok(h) => h,
        Err(err) => return Err(ServiceError::InvalidRequest(err.to_string())),
    };
    match wallet.sign_message(hash).await {
        Ok(s) => {
            let _ = sqlx::query(
                r#"Update bind_code set status = $1, updated_at = now() where id = $2"#,
            )
            .bind(2i16)
            .bind(codes[0].id)
            .execute(&context.db)
            .await?;
            Ok(format!("0x{}", s))
        }
        Err(err) => Err(ServiceError::InvalidRequest(err.to_string())),
    }
}
