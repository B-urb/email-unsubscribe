fn handle_custom_protocol(uri: String) -> Result<String, String> {
  if uri.starts_with("myapp://oauth2?code=") {
    let code = &uri["myapp://oauth2?code=".len()..];

    // TODO: Exchange the code for an access token and ID token.
    return Ok("".to_string());
  }

  Ok("".to_string())
}

async fn oauth_callback(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query().unwrap_or("");

    let code_param = query
        .split('&')
        .find(|p| p.starts_with("code="))
        .unwrap_or("");

    let code = &code_param[5..];

    // TODO: Exchange the code for an access token and ID token.

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Logged in successfully. You may close this window."))
        .unwrap();

    Ok(response)
}