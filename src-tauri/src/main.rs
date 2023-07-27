#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
  )]
//define a global constant for the client id
use hyper::{
  service::{make_service_fn, service_fn},
  Body, Request, Response, Server, StatusCode,
};
use tauri::Manager;
use tokio::{task::JoinHandle, sync::Mutex, sync::mpsc::{channel, Sender}};
use std::{convert::Infallible, collections::HashMap, sync::{Arc}};
use std::net::SocketAddr;
mod mail;
use mail::mail::EmailHandler;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
//create a constant global string
const REDIRECT_URI: &str = "http://localhost:8000";

async fn start_server(addr: SocketAddr, tx: Sender<String>) -> JoinHandle<()> {
  let tx = tx.clone();
  let service = make_service_fn(move |_conn| {
    let tx = tx.clone();
      async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(req,tx.clone()))) }
  });

  let server = Server::bind(&addr).serve(service);

  println!("Listening on http://{}", addr);

  tokio::spawn(async move {
      if let Err(e) = server.await {
          eprintln!("Server error: {}", e);
      }
  })
}

struct loginData {
  token: String,
}
fn parse_query(query: &str) -> HashMap<String, String> {
  query
      .split('&')
      .filter_map(|part| {
          let mut split = part.splitn(2, '=');
          let key = split.next()?.to_string();
          let value = split.next().map(|v| v.to_string())?;
          Some((key, value))
      })
      .collect()
}

async fn handle_request(req: Request<Body>,tx: Sender<String>) -> Result<Response<Body>, Infallible> {
  let mut response = Response::new(Body::from("Hello, World!"));


  if req.uri().path() == "/" {
      if let Some(query) = req.uri().query() {
          let params = parse_query(query);
          println!("Received OAuth redirect with params: {:?}", params);

          // Handle the OAuth redirect here
          // For example, you can get the "code" parameter like this:
          let code = match params.get("code") {
              Some(code) => code,
              None => {
                  *response.status_mut() = StatusCode::BAD_REQUEST;
                  *response.body_mut() = Body::from("Missing query parameters");
                  return Ok(response);
              }
          };
          println!("Code: {:?}", code);
          tx.send(code.clone()).await;
  // Send data to the main thread
  
      } else {
          *response.status_mut() = StatusCode::BAD_REQUEST;
          *response.body_mut() = Body::from("Missing query parameters");
      }
  }

  Ok(response)
}

//Async main function with tokio
#[tokio::main]
async fn main() {
  dotenv::dotenv().ok();
  println!("Starting Tauri application...");
  let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
  let (tx, mut rx) = channel::<String>(100); 
  let server_handle = start_server(addr, tx).await;

  let client_secret: String = match std::env::var("CLIENT_SECRET") {
    Ok(val) => val,
    Err(_) => "CLIENT_ID".to_string(),
    };
    let client_id: String = match std::env::var("CLIENT_ID") {
      Ok(val) => val,
      Err(_) => "CLIENT_ID".to_string(),
      };


  let app = tauri::Builder::default()
  .invoke_handler(tauri::generate_handler![get_client_id])
  .build(tauri::generate_context!())
  .expect("error while building tauri application");

  let app_handle = app.handle();
  let thread_handle = tokio::spawn(async move {
    println!("Starting thread");
    while let Some(item) = rx.recv().await {
      println!("Received: {}", item);
      app_handle.emit_all("redirect-data", item.clone()).expect("Failed to emit event");
      let token = get_access_token(&item, &client_id, &client_secret, "http://localhost:8000").await.unwrap();
      let mut email_handler = EmailHandler::new(token);
      let mails = email_handler.fetch_mails().await; 
      app_handle.emit_all("mail-content", mails).expect("Failed to emit event");
  }

  println!("Sender has been dropped, exiting loop.");
  });

  app.run(|_,_| ());
  thread_handle.abort();
  
}



#[tauri::command]
fn get_client_id() -> String {
  let client_id: String = match std::env::var("CLIENT_ID") {
  Ok(val) => val,
  Err(_) => "CLIENT_ID".to_string(),
  };
   format!("{}",client_id )
}
pub async fn get_access_token(auth_code: &str,
  client_id: &str,
  client_secret: &str,
  redirect_uri: &str,
) -> Result<String, Box<dyn Error>>  {
  let client = reqwest::Client::new();
  let params = [
      ("code", auth_code),
      ("client_id", client_id),
      ("client_secret", client_secret),
      ("redirect_uri", "http://localhost:8000"),
      ("grant_type", "authorization_code"),
  ];

  let response = client
      .post("https://oauth2.googleapis.com/token")
      .form(&params)
      .send()
      .await?;

  let response_text = response.text().await?;
  let response_json: Value = serde_json::from_str(&response_text)?;

  if let Some(access_token) = response_json["access_token"].as_str() {
      Ok(access_token.to_string())
  } else {
      Err(format!("Failed to get access token: {}", response_text).into())
  }
 }
