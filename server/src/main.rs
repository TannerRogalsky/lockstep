use webrtc_unreliable::{Server as RtcServer, SessionEndpoint};

#[derive(Debug)]
struct AppConfig {
    http: std::net::SocketAddr,
    webrtc_data: std::net::SocketAddr,
    webrtc_public: std::net::SocketAddr,
}

impl Default for AppConfig {
    fn default() -> Self {
        let localhost = [127, 0, 0, 1];
        AppConfig {
            http: (localhost, 3030).into(),
            webrtc_data: (localhost, 3030).into(),
            webrtc_public: (localhost, 3030).into(),
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

    let mut hasher = twox_hash::XxHash64::with_seed(0);
    std::hash::Hash::hash(&0, &mut hasher);
    let hash = std::hash::Hasher::finish(&hasher);
    log::info!("hash: {}", hash);

    let config = AppConfig::try_from_env().unwrap_or_default();
    log::info!("config: {:#?}", config);

    let (mut sx, mut rx) = tokio::sync::mpsc::channel(100);
    let mut rtc_server = RtcServer::new(config.webrtc_data, config.webrtc_public)
        .await
        .expect("could not start RTC server");

    let state = std::sync::Arc::new(std::sync::Mutex::new(AppState::with_capacity(60)));
    tokio::spawn({
        let state = std::sync::Arc::clone(&state);
        let dur = std::time::Duration::from_secs_f64(1. / 60.);
        async move {
            loop {
                let r = if let Ok(mut state) = state.lock() {
                    state.step();
                    let last = state.snapshots.back().unwrap();
                    let mut hasher = twox_hash::XxHash64::with_seed(0);
                    std::hash::Hash::hash(&last.state, &mut hasher);
                    let hash = std::hash::Hasher::finish(&hasher);
                    Some((last.frame_index, hash))
                } else {
                    None
                };
                if let Some((frame_index, hash)) = r {
                    if let Err(err) = sx.send((frame_index, hash)).await {
                        log::error!("{}", err);
                    }
                    tokio::time::delay_for(dur).await;
                }
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
            let hash_get = warp::get().and(warp::path("hash")).map(|| {
                let state = shared::SharedState::new(16, 1. / 60.);
                let hash = state.hash();
                warp::reply::json(&(hash, state))
            });
            warp::serve(public.or(rtc).or(state_get).or(hash_get))
                .run(config.http)
                .await;
        }
    });

    async fn on_internal_message(
        rtc_server: &mut RtcServer,
        (frame_index, hash): (shared::FrameIndex, u64),
    ) {
        let msg = shared::Recv::StateHash(shared::IndexedState {
            frame_index,
            state: hash,
        });
        let msg = bincode::serialize(&msg).unwrap();
        let connected_clients = rtc_server.connected_clients().copied().collect::<Vec<_>>();
        for connected_client in connected_clients {
            if let Err(err) = rtc_server
                .send(
                    &msg,
                    webrtc_unreliable::MessageType::Binary,
                    &connected_client,
                )
                .await
            {
                log::error!("{}", err);
            }
        }
    }

    async fn on_external_message(
        rtc_server: &mut RtcServer,
        message_buf: &mut Vec<u8>,
        state: &std::sync::Arc<std::sync::Mutex<AppState>>,
        message: Option<(webrtc_unreliable::MessageType, std::net::SocketAddr)>,
    ) {
        if let Some((message_type, remote_addr)) = message {
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

    async fn try_external(
        rtc_server: &mut RtcServer,
        message_buf: &mut Vec<u8>,
    ) -> Option<(webrtc_unreliable::MessageType, std::net::SocketAddr)> {
        match rtc_server.recv().await {
            Ok(received) => {
                message_buf.clear();
                message_buf.extend(received.message.as_ref());
                Some((received.message_type, received.remote_addr))
            }
            Err(err) => {
                log::warn!("could not receive RTC message: {}", err);
                None
            }
        }
    }

    let mut message_buf = Vec::new();
    loop {
        tokio::select! {
            message = rx.recv() => {
                on_internal_message(&mut rtc_server, message.unwrap()).await;
            },
            message = try_external(&mut rtc_server, &mut message_buf) => {
                on_external_message(&mut rtc_server, &mut message_buf, &state, message).await;
            }
        }
    }
}
