pub mod mail {
use imap::{connect, Client};
use base64;
use mailparse::*;


    extern crate native_tls;

    struct GmailOAuth2 {
        user: String,
        access_token: String,
    }
    
    impl imap::Authenticator for GmailOAuth2 {
        type Response = String;
        fn process(&self, _: &[u8]) -> Self::Response {
            format!(
                "user={}\x01auth=Bearer {}\x01\x01",
                self.user, self.access_token
            )
        }
    }
    
    pub struct EmailHandler {
        pub token: String,
    }
    
    impl EmailHandler {
        pub fn  new(
            token: String,
        ) -> Self {
            EmailHandler {
                token,
            } 
           }

  

           pub async fn fetch_mails(&mut self) -> String  {
            // Google Mail IMAP server address and port
            let gmail_auth = GmailOAuth2 {
                user: String::from("bjoclurban@gmail.com"),
                access_token: String::clone(&self.token),
            };
            println!("{}", gmail_auth.access_token);
          
            let domain = "imap.gmail.com";
            let tls = native_tls::TlsConnector::builder().build().unwrap();
            let client = imap::connect((domain, 993), domain, &tls).unwrap();
            let mut session = match client.authenticate("XOAUTH2", &gmail_auth) {
                Ok(session) => session,       
                Err((e, orig_client)) => {
                    eprintln!("error authenticating: {}", e);
                    return String::from("error authenticating");
                }
            };
           // match client.login(gmail_auth.user, gmail_auth.access_token) {
            //    Ok(ses) => session = ses,
             //   Err((e, orig_client)) => eprintln!("Error logging in: {}", e),
            //};

            match session.select("INBOX") {
                Ok(_) => println!("Selected INBOX"),
                Err(e) => eprintln!("Error selecting INBOX: {}", e),
            };

            // fetch message number 1 in this mailbox, along with its RFC822 field.
            // RFC 822 dictates the format of the body of e-mails
            let messages = match session.fetch("1", "RFC822") {
                Ok(me) => me,
                Err(e) => {
                    eprintln!("Error fetching message: {}", e);
                    return e.to_string();
                }
            };
            let message = if let Some(m) = messages.iter().next() {
                m
            } else {
                return String::from("No messages in INBOX");
            };
        
            // extract the message's body
            let body = message.body().expect("message did not have a body!");
            //let body = std::str::from_utf8(body)
              //  .expect("message was not valid utf-8")
                //.to_string();
            let parsed = parse_mail(body)
                .unwrap();
            println!("{}", parsed.get_body().unwrap());
                    
        session.logout().unwrap();
        return parsed.get_body().unwrap();
           }
        }
    }