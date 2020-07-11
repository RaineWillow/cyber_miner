mod api;
mod asm;
mod config;
mod robot;

use crate::config::{Config, SecureConfig};
use cookie::{Cookie, CookieJar, Key};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, SinkExt, StreamExt};
use http::header::{HeaderValue, COOKIE, SET_COOKIE};
use http::status::StatusCode;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::handshake::server::{
    Callback, ErrorResponse, Request, Response,
};
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;

#[derive(Clone)]
struct ServerState {
    peer_map: Arc<Mutex<HashMap<SocketAddr, Tx>>>,
    cookie_jar: ThreadPrivateJar,
}

impl ServerState {
    fn new(key: Key) -> Self {
        let cookie_jar = ThreadPrivateJar::new(key);
        Self {
            peer_map: Arc::new(Mutex::new(HashMap::new())),
            cookie_jar,
        }
    }
}

// Sets which cookie key we want to use
const COOKIE_NAME_ID: &str = "id";

/// Used to store cookies across threads
#[derive(Clone)]
struct ThreadPrivateJar {
    cur_id: usize,
    key: Key,
    jar: Arc<Mutex<CookieJar>>,
}

impl ThreadPrivateJar {
    fn new(key: Key) -> Self {
        // Create a cookie jar
        let jar = CookieJar::new();

        Self {
            cur_id: 0,
            key,
            jar: Arc::new(Mutex::new(jar)),
        }
    }
    fn new_cookie_response(&mut self, response: Response) -> Result<Response, ErrorResponse> {
        // Construct a cookie
        let cookie = Cookie::build(COOKIE_NAME_ID, self.cur_id.to_string())
            // TODO: set secure to true once we're on https
            //.secure(true)
            .finish();
        debug!("Created new cookie {}", cookie);
        // Increment the id
        self.cur_id += 1;
        // Open the cookie jar for writing
        // Shadow cookie since it is being replaced
        let cookie = match self.jar.lock() {
            Ok(mut jar) => {
                // Open the jar using the private key
                let mut private_jar = jar.private(&self.key);
                // Store our cookie in the private jar
                private_jar.add_original(cookie);
                // Retrieve the cookie from the regular jar after encrypting it in the private
                // one. Then, convert it to a string
                match jar.get(COOKIE_NAME_ID) {
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
        };
        // Convert cookie to string
        let cookie_str = cookie.to_string();
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

impl Callback for &mut ThreadPrivateJar {
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
                            // Check to see if the cookie sent is authenticated with our key
                            match jar.lock() {
                                Ok(mut jar) => {
                                    // Open the jar using the private key
                                    let private_jar = jar.private(&self.key);
                                    if let Some(cookie) = private_jar.get(COOKIE_NAME_ID) {
                                        debug!("Cookie received decrypts to {}", cookie);
                                        Ok(response)
                                    }
                                    // Error validating cookie, make a new one
                                    else {
                                        warn!("Received invalid cookie from client. Giving them a new one");
                                        self.new_cookie_response(response)
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
                            self.new_cookie_response(response)
                        }
                    }
                }
                // Failed to convert cookie header to string
                Err(err) => {
                    error!("Error converting cookie header value to string: {}", err);
                    self.new_cookie_response(response)
                }
            }
        }
        // No cookie was provided, make a new one
        else {
            self.new_cookie_response(response)
        }
    }
}

async fn handle_connection(mut state: ServerState, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, &mut state.cookie_jar)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    //let (tx, rx) = unbounded();
    //state.peer_map.lock().unwrap().insert(addr, tx);

    let (mut outgoing, incoming) = ws_stream.split();

    let print_incoming = incoming.try_for_each(|msg| {
        info!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        if let Message::Ping(data) = msg {
            outgoing.send(Message::Pong(data));
        }
        future::ok(())
    });

    print_incoming
        .await
        .expect("Failed to handle incoming messages");
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
