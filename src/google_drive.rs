use anyhow::Result;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl, basic::BasicClient,
};
use tokio::net::TcpListener;

async fn login() -> Result<()> {
    // OS picks a free port for us
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let redirect_url = format!("http://{}/callback", addr);

    let client = BasicClient::new(ClientId::new("CLIENT_ID".to_string()))
        .set_client_secret(ClientSecret::new("CLIENT_SECRET".to_string()))
        .set_auth_uri(AuthUrl::new(
            "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://oauth2.googleapis.com/token".to_string(),
        )?)
        .set_redirect_uri(RedirectUrl::new(redirect_url)?);

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/drive.file".to_string(),
        ))
        .url();

    println!("{auth_url}");
    webbrowser::open(auth_url.as_str())?;

    // next: use the listener to receive the callback code
    Ok(())
}
