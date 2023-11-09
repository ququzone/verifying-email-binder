use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use sqlx::PgPool;
use tracing::{error, info};

use crate::service::code::BindCode;

pub async fn send_mails(db: &PgPool, key: &str, from: &str, host: &str) {
    let codes = sqlx::query_as::<_, BindCode>(
        "select id, account, email, code, status, created_at, updated_at from bind_code where status = 0 order by id desc limit 100",
    ).fetch_all(db).await;

    match codes {
        Ok(codes) => {
            for code in codes {
                let email: Message = Message::builder()
                    .from(from.parse().unwrap())
                    .to(code.email.parse().unwrap())
                    .subject(format!("ioPay AA Wallet Verification Code - {}", code.code))
                    .body(
format!("Dear User,

I hope this message finds you well. We are writing to provide you with an important piece of information regarding your ioPay AA Wallet.

Your unique verification code is: 
{}

This code is essential for the verification process of your wallet and should be entered in the required field to proceed.

We strongly advise you to keep this code confidential. It is crucial to the security of your wallet and should not be shared with anyone under any circumstances. Your privacy and security are our top priorities, and we want to ensure that your wallet remains secure at all times.

Please remember to store this code in a safe and secure place where only you can access it. If you suspect that your code has been compromised, please contact our support team immediately.

Thank you for your attention to this matter. We appreciate your cooperation in maintaining the security of your ioPay AA Wallet.

Best Regards,
ioPay Team", code.code))
                    .unwrap();

                let creds: Credentials = Credentials::new(from.to_string(), key.to_string());
                let mailer: SmtpTransport = SmtpTransport::relay(host)
                    .unwrap()
                    .credentials(creds)
                    .build();
                match mailer.send(&email) {
                    Ok(_) => {
                        let _ = sqlx::query(
                            r#"Update bind_code set status = $1, updated_at = now() where id = $2"#,
                        )
                        .bind(1i16)
                        .bind(code.id)
                        .execute(db)
                        .await;
                        info!(target: "email", id = ?code.id, email = ?code.email, "send email success")
                    }
                    Err(err) => {
                        println!("Send email error: {}", err);
                        error!(target: "email", id = ?code.id, email = ?code.email, err = ?err, "send email")
                    }
                };
            }
        }
        Err(err) => {
            println!("Query codes error: {}", err);
            error!(target: "email", ?err, "query codes error")
        }
    }
}
