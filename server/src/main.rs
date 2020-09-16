use webrtc_unreliable::{Server as RtcServer, SessionEndpoint};

#[derive(Debug)]
struct AppConfig {
    http: std::net::SocketAddr,
    webrtc_data: std::net::SocketAddr,
    webrtc_public: std::net::SocketAddr,
    udp: std::net::SocketAddr,
}

impl Default for AppConfig {
    fn default() -> Self {
        let localhost = [127, 0, 0, 1];
        AppConfig {
            http: (localhost, 3030).into(),
            webrtc_data: (localhost, 3030).into(),
            webrtc_public: (localhost, 3030).into(),
            udp: (localhost, 43434).into(),
        }
    }
}

impl AppConfig {
    pub fn try_from_env() -> Option<Self> {
        let port = match std::env::var("PORT").map(|s| s.parse::<u16>()) {
            Ok(Ok(port)) => port,
            _ => return None,
        };

        let binding = [0, 0, 0, 0];
        Some(AppConfig {
            http: (binding, port).into(),
            webrtc_data: (binding, port).into(),
            webrtc_public: (binding, port).into(),
            udp: (binding, port).into(),
        })
    }
}

struct AppState {
    snapshots: std::collections::VecDeque<shared::IndexedState<Vec<u8>>>,
    current: shared::State<()>,
    input_state: shared::InputState,
}

impl AppState {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            snapshots: std::collections::VecDeque::with_capacity(capacity),
            current: shared::State::new(()),
            input_state: shared::InputState::default(),
        }
    }

    pub fn step(&mut self) {
        let snapshot = bincode::serialize(&self.current.physics_state).unwrap();
        if self.snapshots.len() == self.snapshots.capacity() {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(shared::IndexedState {
            frame_index: self.current.frame_index,
            state: snapshot,
        });
        self.current.step(self.input_state)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let config = AppConfig::try_from_env().unwrap_or_default();
    log::info!("config: {:#?}", config);

    let mut rtc_server = RtcServer::new(config.webrtc_data, config.webrtc_public)
        .await
        .expect("could not start RTC server");

    let udp_server = tokio::net::UdpSocket::bind(&config.udp)
        .await
        .expect("couldn't bind udp socket");

    tokio::spawn(async move {
        let mut socket = udp_server;
        let mut buf = vec![0; 1024];
        let mut to_send = None;

        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            if let Some((size, peer)) = to_send {
                let amt = socket.send_to(&buf[..size], &peer).await.unwrap();

                log::debug!("Echoed {}/{} bytes to {}", amt, size, peer);
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            to_send = Some(socket.recv_from(&mut buf).await.unwrap());
        }
    });

    let state = std::sync::Arc::new(std::sync::Mutex::new(AppState::with_capacity(60)));
    std::thread::spawn({
        let state = std::sync::Arc::clone(&state);
        let dur = std::time::Duration::from_secs_f64(0.01666666666);
        move || loop {
            if let Ok(mut state) = state.lock() {
                state.step();
                drop(state);
                std::thread::sleep(dur);
            }
        }
    });

    let session_endpoint = rtc_server.session_endpoint();
    tokio::spawn({
        let state = std::sync::Arc::clone(&state);
        async move {
            use warp::Filter;

            async fn rtc_callback<S, B>(
                req: S,
                mut session_endpoint: SessionEndpoint,
            ) -> Result<warp::reply::Response, warp::Rejection>
            where
                B: bytes::Buf,
                S: futures::Stream<Item = Result<B, warp::Error>>,
            {
                use futures::TryStreamExt;
                use warp::Reply;

                #[derive(Debug)]
                struct SessionErrorWrapper<T>(T);

                impl<T> warp::reject::Reject for SessionErrorWrapper<T> where
                    T: std::fmt::Debug + Sized + Send + Sync + 'static
                {
                }

                match session_endpoint
                    .http_session_request(req.map_ok(|mut buf| buf.to_bytes()))
                    .await
                {
                    Ok(resp) => Ok(warp::reply::with_header(
                        resp,
                        warp::hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        "*",
                    )
                    .into_response()),
                    Err(err) => Err(warp::reject::custom(SessionErrorWrapper(err))),
                }
            }

            let public = warp::fs::dir(std::path::Path::new("server").join("public"));
            let rtc = warp::post()
                .and(warp::path("new_rtc_session"))
                .and(warp::body::stream())
                .and(warp::any().map(move || session_endpoint.clone()))
                .and_then(rtc_callback);
            let state_get = warp::get().and(warp::path("state")).map(move || {
                let state = state.lock().unwrap();
                let output = shared::IndexedState {
                    frame_index: state.current.frame_index,
                    state: &state.current.physics_state,
                };
                warp::Reply::into_response(bincode::serialize(&output).unwrap())
            });
            warp::serve(public.or(rtc).or(state_get))
                .run(config.http)
                .await;
        }
    });

    let mut message_buf = Vec::new();
    loop {
        let received = match rtc_server.recv().await {
            Ok(received) => {
                message_buf.clear();
                message_buf.extend(received.message.as_ref());
                Some((received.message_type, received.remote_addr))
            }
            Err(err) => {
                log::warn!("could not receive RTC message: {}", err);
                None
            }
        };

        if let Some((message_type, remote_addr)) = received {
            let response = match bincode::deserialize::<shared::Send>(&message_buf) {
                Err(err) => {
                    log::error!("{}", err);
                    None
                }
                Ok(shared::Send::Ping(frame_index)) => Some(shared::Recv::Pong(frame_index)),
                Ok(shared::Send::InputState(input_state)) => {
                    let mut state = state.lock().unwrap();
                    state.input_state = input_state.state;
                    None
                }
            };
            if let Some(response) = response {
                let message_buf = bincode::serialize(&response).unwrap();
                match rtc_server
                    .send(&message_buf, message_type, &remote_addr)
                    .await
                {
                    Ok(_) => log::trace!("send buf success to {}: {:?}", remote_addr, message_buf),
                    Err(err) => log::warn!("could not send message to {}: {}", remote_addr, err),
                }
            }
        }
    }
}
