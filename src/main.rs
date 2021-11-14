#![allow(warnings)]
use reqwest::{blocking::Client, header::CONTENT_TYPE};
use std::{
    error::Error,
    io::{self, Write},
};
use structopt::StructOpt;

trait ToIoError
where
    Self: Sized + Send + Sync + 'static,
    Self: Into<Box<(dyn std::error::Error + Send + Sync + 'static)>>,
{
    fn to_error(self) -> io::Error;
}

impl<T> ToIoError for T
where
    T: Sized + Send + Sync + 'static,
    T: Into<Box<(dyn std::error::Error + Send + Sync + 'static)>>,
{
    fn to_error(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self.into())
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "restaff-claim-points", about = "Claim point from Restaff page")]
pub struct Args {
    #[structopt(short, long, help = "It's just your username, what can I say ;)")]
    username: Option<String>,

    #[structopt(
        short,
        long,
        help = "Only specify user to open a prompt to input password"
    )]
    password: Option<String>,

    #[structopt(short, long, default_value = "3", help = "Claim type, from 1 to 5")]
    claim_type: u8,

    #[structopt(
        short,
        long,
        default_value = "https://api-staff.netjob.asia",
        help = "Specify API server"
    )]
    api_server: String,

    #[structopt(short = "f", long, help = "Use password file (Base64 encoded)")]
    password_file: Option<String>,

    #[structopt(short, long, help = "Use JWT token")]
    token: Option<String>,

    #[structopt(short = "k", long, help = "Use JWT token file")]
    token_file: Option<String>,
}

type Token = String;

const RESTAFF_API_LOGIN: &str = "/api/user/login";
const RESTAFF_API_CLAIM: &str = "/api/user/claim-daily";
const RESTAFF_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:88.0) Gecko/20100101 Firefox/88.0";

fn restaff_login(api_server: &str, username: &str, password: &str) -> anyhow::Result<Token> {
    let client = Client::builder().user_agent(RESTAFF_USER_AGENT).build()?;

    let body = format!(
        r#"{{"UserName":"{}","PassWord":"","Password":"{}","pushToken":"","platform":"Web","deviceId":"Windows-Firefox-88.0"}}"#,
        username, password
    );

    let request = client
        .post(format!("{}{}", api_server, RESTAFF_API_LOGIN))
        .header("appid", "c03714075869519a54ba70e31d6751c3")
        .header(CONTENT_TYPE, "application/json")
        .body(body);

    let response = request.send()?;
    let response_string = response.text()?;
    let json = serde_json::from_str::<serde_json::Value>(&response_string)?;

    let code = json
        .get("code")
        .and_then(|code| code.as_i64())
        .ok_or_else(|| "Bad response (contains no `id`)".to_error())?;

    if code < 0 {
        Err("Login failed".to_error())?;
    }

    let token = json
        .get("data")
        .and_then(|data| data.get("token"))
        .and_then(|token| token.as_str())
        .ok_or_else(|| "Response doesn't contain token".to_error())?;

    Ok(token.to_string())
}

fn restaff_claim_points(api_server: &str, token: &str, claim_type: u8) -> anyhow::Result<i64> {
    let client = Client::new();

    let response = client
        .post(format!("{}{}", api_server, RESTAFF_API_CLAIM))
        .header("appid", "c03714075869519a54ba70e31d6751c3")
        .header(CONTENT_TYPE, "application/json")
        .body(claim_type.to_string())
        .bearer_auth(token)
        .send()?
        .text()?;

    let id_response = serde_json::from_str::<serde_json::Value>(&response)?
        .get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| "Invalid id value from response".to_error())?;

    Ok(id_response)
}

fn split_once<'a>(src: &'a str, pat: &str) -> Option<(&'a str, &'a str)> {
    src.find(pat).map(|pos| (&src[..pos], &src[pos + 1..]))
}

fn acquire_token(mut args: Args) -> Option<String> {
    let mut username = args.username.unwrap_or(String::new());

    let password = if let Some(pw) = args.password {
        pw
    } else if let Some(pf) = args.password_file {
        let content = match std::fs::read_to_string(&pf) {
            Ok(s) => s,
            Err(err) => {
                println!("Error: read content from file `{}`: {}", pf, err);
                return None;
            }
        };

        let mut content = content.trim();

        match split_once(content, ":") {
            Some((un, pw)) => {
                username = un.to_string();
                content = pw;
            }
            None => {}
        }

        let decoded = match base64::decode(&content) {
            Ok(pw) => pw,
            Err(err) => {
                println!("Error: decoding password: {}", err);
                return None;
            }
        };

        let pw = String::from_utf8_lossy(&decoded).to_string();
        pw
    } else {
        if username.is_empty() {
            eprintln!("Error: username is empty");
            return None;
        }

        print!("Input password: ");
        io::stdout().flush();
        match rpassword::read_password() {
            Ok(pw) => pw,
            Err(err) => {
                eprintln!(
                    "Error: can't read password from console. Try input password via option `-p`"
                );
                return None;
            }
        }
    };

    if username.is_empty() {
        eprintln!("Error: username is empty");
        return None;
    }

    if !matches!(args.claim_type, 0..=5) {
        args.claim_type = 3;
    }

    let token = match restaff_login(&args.api_server, username.as_str(), password.as_str()) {
        Ok(token) => token,
        Err(err) => {
            println!("Error: {}", err);
            return None;
        }
    };

    Some(token)
}

fn main() {
    let mut args = Args::from_args();

    let claim_type = args.claim_type;
    let api_server = args.api_server.to_string();
    let mut is_using_input_token = true;

    let token = if let Some(token) = args.token {
        is_using_input_token = false;
        token
    } else if let Some(token_file) = args.token_file {
        is_using_input_token = false;
        match std::fs::read_to_string(&token_file) {
            Ok(content) => content.trim().to_string(),
            Err(err) => {
                eprintln!("Error: Failed to read token file content: {}", token_file);
                return;
            }
        }
    } else if let Some(token) = acquire_token(args) {
        println!("Login successfully.");
        token
    } else {
        println!("Login failed.");
        return;
    };

    match restaff_claim_points(api_server.as_str(), token.as_str(), claim_type) {
        Ok(id) if id >= 0 => println!("Points claimed successfully."),
        Ok(id) => println!("Points claimed failed. Returned id: {}", id),
        Err(err) => {
            if is_using_input_token {
                eprintln!("Error: Used user input token and failed.");
            }
            eprintln!("Error: {}", err);
        }
    }
}
