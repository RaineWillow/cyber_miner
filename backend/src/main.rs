mod api;
mod asm;
mod config;
mod robot;
mod user;

use crate::api::{CodeError, Request as ApiRequest, Response as ApiResponse};
use crate::asm::AssemblyLine;
use crate::config::{Config, SecureConfig};
use crate::user::User;
use cookie::{Cookie, CookieJar, Key};
use futures_channel::mpsc::UnboundedSender;
use futures_util::{SinkExt, StreamExt};
use http::header::{HeaderValue, COOKIE, SET_COOKIE};
use http::status::StatusCode;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::handshake::server::{
    Callback, ErrorResponse, Request, Response,
};
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;

#[derive(Clone)]
struct ServerState {
    peer_map: Arc<Mutex<HashMap<SocketAddr, Tx>>>,
    cookie_jar: Arc<ThreadPrivateJar>,
    // TODO: make this fnv hashmap
    // TODO: better sync primitive
    users: Arc<RwLock<HashMap<usize, User>>>,
}

impl ServerState {
    fn new(key: Key) -> Self {
        Self {
            peer_map: Arc::new(Mutex::new(HashMap::new())),
            cookie_jar: Arc::new(ThreadPrivateJar::new(key)),
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Used to store cookies across threads
struct ThreadPrivateJar {
    cur_id: AtomicUsize,
    key: Key,
    jar: Mutex<CookieJar>,
}

impl ThreadPrivateJar {
    fn new(key: Key) -> Self {
        // Create a cookie jar
        let jar = CookieJar::new();

        Self {
            cur_id: AtomicUsize::new(0),
            key,
            jar: Mutex::new(jar),
        }
    }
}

/// Stores a reference to the big cookig jar, and
/// has a value for storing this connection's cookie
struct CookieHandler {
    conn_id: Option<usize>,
    jar: Arc<ThreadPrivateJar>,
}
impl CookieHandler {
    fn new(jar: Arc<ThreadPrivateJar>) -> Self {
        Self { conn_id: None, jar }
    }
}

impl CookieHandler {
    fn new_cookie_response(
        &mut self,
        response: Response,
        lock: Option<MutexGuard<CookieJar>>,
    ) -> Result<Response, ErrorResponse> {
        // Increment the user id and return the old one
        // Allocate a new user id
        let my_id = self.jar.cur_id.fetch_add(1, Ordering::Relaxed);
        // Store the id for this session
        self.conn_id = Some(my_id);
        // Construct a cookie
        let new_cookie_name = format!("user_{}", my_id);
        // TODO: set the value to some random stuff
        let new_cookie = Cookie::build(new_cookie_name.clone(), my_id.to_string())
            // TODO: set secure to true once we're on https
            //.secure(true)
            .finish();
        debug!("Created new cookie {}", new_cookie);
        let cookie_str = {
            // Open the cookie jar for writing
            let lock = match lock {
                Some(lock) => Ok(lock),
                None => self.jar.jar.lock(),
            };
            match lock {
                Ok(mut jar) => {
                    // Open the jar using the private key
                    let mut private_jar = jar.private(&self.jar.key);
                    // Store our cookie in the private jar
                    private_jar.add_original(new_cookie);
                    // Retrieve the cookie from the regular jar after encrypting it in the private
                    // one. Then, convert it to a string
                    match jar.get(&new_cookie_name) {
                        Some(cookie) => cookie.to_string(),
                        None => {
                            error!("Error retrieving cookie just added to cookie jar");
                            return Err(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(None)
                                .unwrap());
                        }
                    }
                }
                Err(err) => {
                    error!("Error getting access to cookie jar: {}", err);
                    return Err(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(None)
                        .unwrap());
                }
            }
        };
        // Convert cookie string to header
        match HeaderValue::from_str(&cookie_str) {
            Ok(cookie_header) => {
                let mut response = response;
                response.headers_mut().insert(SET_COOKIE, cookie_header);
                Ok(response)
            }
            Err(err) => {
                error!("Error generating cookie header: {}", err);
                return Err(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(None)
                    .unwrap());
            }
        }
    }
}

impl Callback for &mut CookieHandler {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        // Get one reference to the jar
        let jar = self.jar.clone();

        // Check if there's a cookie in the request
        if let Some(cookie) = request.headers().get(COOKIE) {
            // Convert the header to a string
            match cookie.to_str() {
                Ok(cookie) => {
                    // Parse the cookie
                    match Cookie::parse(cookie) {
                        Ok(cookie) => {
                            debug!("Received request with cookie {} from client", cookie);
                            // Store the cookie's name
                            let cookie_name = cookie.name();
                            // Check to see if the cookie sent is authenticated with our key
                            match jar.jar.lock() {
                                Ok(mut jar) => {
                                    // Open the jar using the private key
                                    let private_jar = jar.private(&self.jar.key);
                                    if let Some(jar_cookie) = private_jar.get(&cookie_name) {
                                        debug!(
                                            "Cookie in jar for {} is {}",
                                            cookie_name,
                                            jar_cookie.value()
                                        );
                                        // Parse out the cookie as an id
                                        match jar_cookie.value().parse() {
                                            Ok(id) => {
                                                self.conn_id = Some(id);
                                                Ok(response)
                                            }
                                            Err(err) => {
                                                error!("Error parsing id as integer: {}", err);
                                                Err(Response::builder()
                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                    .body(None)
                                                    .unwrap())
                                            }
                                        }
                                    }
                                    // Error validating cookie, make a new one
                                    else {
                                        warn!("Received invalid cookie from client. Giving them a new one");
                                        self.new_cookie_response(response, Some(jar))
                                    }
                                }
                                // Error getting access to the rwlock
                                Err(err) => {
                                    error!("Error getting access to cookie jar: {}", err);
                                    Err(Response::builder()
                                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                                        .body(None)
                                        .unwrap())
                                }
                            }
                        }
                        // Failed to parse cookie header value as cookie
                        Err(err) => {
                            error!("Error parsing cookie: {}", err);
                            self.new_cookie_response(response, None)
                        }
                    }
                }
                // Failed to convert cookie header to string
                Err(err) => {
                    error!("Error converting cookie header value to string: {}", err);
                    self.new_cookie_response(response, None)
                }
            }
        }
        // No cookie was provided, make a new one
        else {
            self.new_cookie_response(response, None)
        }
    }
}

async fn handle_connection(mut state: ServerState, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);
    // Get jar
    let jar = state.cookie_jar.clone();
    // Create a cookie handler for this session
    let mut cookie_handler = CookieHandler::new(jar);
    // Initialize the connection and extract or create the cookies
    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, &mut cookie_handler)
        .await
        .expect("Error during the websocket handshake occurred");
    // Get the user id
    let user_id = if let Some(user_id) = cookie_handler.conn_id {
        user_id
    } else {
        error!("User connected without an id");
        return;
    };
    info!("User logged in from {} with id {}", addr, user_id);

    let (mut outgoing, mut incoming) = ws_stream.split();

    while let Some(message) = incoming.next().await {
        match message {
            Ok(Message::Text(message_text)) => {
                // Parse the message
                let message = serde_json::from_str(&message_text);
                // Handle the code
                match message {
                    Ok(ApiRequest::UploadCode(code)) => {
                        // Extract into parts for a request
                        let (_code, response) = match asm::parse_code(code) {
                            Ok(code) => (
                                Some(code),
                                ApiResponse::UploadCode {
                                    success: true,
                                    errors: None,
                                },
                            ),
                            Err(errors) => (
                                None,
                                ApiResponse::UploadCode {
                                    success: false,
                                    errors: Some(errors),
                                },
                            ),
                        };
                        // Serialize response as json
                        let response_text = serde_json::to_string(&response).unwrap();
                        // Send response
                        if let Err(err) = outgoing.send(Message::Text(response_text)).await {
                            error!("Error sending response to client: {}", err)
                        }
                    }
                    Err(err) => {
                        error!(
                            "Error parsing message {:?} as request: {}",
                            message_text, err
                        );
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                warn!("Received unhandled binary data: {:?}", data);
            }
            Ok(Message::Ping(data)) => {
                if let Err(err) = outgoing.send(Message::Pong(data)).await {
                    error!("Error sending pong: {}", err);
                }
            }
            Ok(Message::Pong(_data)) => {
                // Do nothing
            }
            Ok(Message::Close(_)) => {
                return;
            }
            Err(err) => {
                error!("Error reading incoming message: {}", err);
            }
        }
    }
    info!("{} disconnected", &addr);
    //state.peer_map.as_ref().lock().unwrap().remove(&addr);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();
    // Parse out the host address
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Load configuration
    const CONFIG_PATH: &str = "config.toml";
    let config: SecureConfig = Config::load(CONFIG_PATH).unwrap_or_default().try_into()?;
    // Save the configuration in case something changed like a key generation
    Config::from(&config).save(CONFIG_PATH)?;

    // Load the cookie key
    let cookie_key = config.get_cookie_key();

    // Create an object for shared state
    let state = ServerState::new(cookie_key);

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let mut listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}
