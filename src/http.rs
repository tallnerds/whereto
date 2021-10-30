pub async fn resolve_url(s: String) -> Result<String, reqwest::Error> {
  let response = reqwest::get(s).await?.status();

  Ok(format!("{:?}", response.to_string()))
}
