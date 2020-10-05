use std::net::SocketAddr;
use tokio::sync::{mpsc, watch};
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
    current: shared::State,
    input_recver: mpsc::UnboundedReceiver<shared::IndexedState<shared::AddBodyEvent>>,
    state_sender: watch::Sender<shared::State>,
}

impl AppState {
    pub fn new(
        input_recver: mpsc::UnboundedReceiver<shared::IndexedState<shared::AddBodyEvent>>,
    ) -> (Self, watch::Receiver<shared::State>) {
        let (state_sender, recver) = watch::channel(Default::default());

        use shared::nbody::{Body, Float};
        let mut current = shared::State::new();
        current.simulation.add_body(Body::new_lossy(0., 0., 10000.));
        current.simulation.add_body({
            let mut body = Body::new_lossy(0., -100., 10.);
            body.velocity.x = Float::from_num(3);
            body
        });
        current.simulation.add_body({
            let mut body = Body::new_lossy(0., 100., 10.);
            body.velocity.x = Float::from_num(-3);
            body
        });

        (
            Self {
                current,
                input_recver,
                state_sender,
            },
            recver,
        )
    }

    pub fn step(&mut self) -> Result<(), watch::error::SendError<shared::State>> {
        while let Ok(input) = self.input_recver.try_recv() {
            log::debug!("EVENT: {:?}", input.state);
            self.current.input_buffer.push(input);
        }
        self.current.step();
        self.state_sender.broadcast(self.current.clone())
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

    let (input_sender, input_recver) = mpsc::unbounded_channel();
    let (mut state, mut state_recver) = AppState::new(input_recver);
    tokio::spawn({
        let dur = std::time::Duration::from_secs_f64(1. / 60.);
        async move {
            loop {
                if let Err(err) = state.step() {
                    log::error!("{}", err);
                }
                tokio::time::delay_for(dur).await;
            }
        }
    });

    let session_endpoint = rtc_server.session_endpoint();
    tokio::spawn({
        let state_recver = state_recver.clone();
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
                let output = state_recver.borrow();
                let hash = output.hash();
                let ser = bincode::serialize(&(hash, &*output)).unwrap();
                warp::Reply::into_response(ser)
            });
            warp::serve(public.or(rtc).or(state_get))
                .run(config.http)
                .await;
        }
    });

    #[cfg(feature = "viz")]
    std::thread::spawn({
        let state_recver = state_recver.clone();
        move || {
            use glutin::platform::windows::EventLoopExtWindows;
            use renderer::*;

            let event_loop = glutin::event_loop::EventLoop::<()>::new_any_thread();
            let size = glutin::dpi::PhysicalSize::new(720, 480);
            let wb = glutin::window::WindowBuilder::new().with_inner_size(size);
            let window = glutin::ContextBuilder::new()
                .with_multisampling(16)
                .build_windowed(wb, &event_loop)
                .unwrap();
            let window = unsafe { window.make_current().unwrap() };

            let glow_ctx = unsafe {
                graphics::glow::Context::from_loader_function(|name| window.get_proc_address(name))
            };
            let context = graphics::Context::new(glow_ctx);
            let mut renderer = renderer::Renderer::new(context, size.width, size.height).unwrap();

            event_loop.run(move |event, _, control_flow| {
                use glutin::{event::*, event_loop::*};
                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id,
                    } if window_id == window.window().id() => match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },
                        WindowEvent::Resized(new_inner_size) => {
                            let glutin::dpi::PhysicalSize { width, height } = *new_inner_size;
                            renderer.resize(width, height);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            let glutin::dpi::PhysicalSize { width, height } = **new_inner_size;
                            renderer.resize(width, height);
                        }
                        _ => {}
                    },
                    Event::RedrawRequested(_) => {
                        let state = state_recver.borrow();
                        renderer.render(&*state);
                        drop(state);
                        window.swap_buffers().unwrap();
                    }
                    Event::MainEventsCleared => {
                        // RedrawRequested will only trigger once, unless we manually request it.
                        window.window().request_redraw();
                    }
                    _ => {}
                }
            });
        }
    });

    async fn broadcast_except(rtc_server: &mut RtcServer, message: &[u8], except: SocketAddr) {
        let connected_clients = rtc_server.connected_clients().copied().collect::<Vec<_>>();
        for connected_client in connected_clients {
            if connected_client != except {
                if let Err(err) = rtc_server
                    .send(
                        message,
                        webrtc_unreliable::MessageType::Binary,
                        &connected_client,
                    )
                    .await
                {
                    log::error!("{}", err);
                }
            }
        }
    }

    async fn broadcast(rtc_server: &mut RtcServer, message: &[u8]) {
        let connected_clients = rtc_server.connected_clients().copied().collect::<Vec<_>>();
        for connected_client in connected_clients {
            if let Err(err) = rtc_server
                .send(
                    message,
                    webrtc_unreliable::MessageType::Binary,
                    &connected_client,
                )
                .await
            {
                log::error!("{}", err);
            }
        }
    }

    async fn on_internal_message(rtc_server: &mut RtcServer, state: shared::State) {
        let hash = state.hash();
        let msg = shared::Recv::StateHash(shared::IndexedState {
            frame_index: state.frame_index,
            state: hash,
        });
        log::trace!("{}, {:?}", hash, state);
        let msg = bincode::serialize(&msg).unwrap();
        broadcast(rtc_server, &msg).await;
    }

    async fn on_external_message(
        rtc_server: &mut RtcServer,
        message_buf: &mut Vec<u8>,
        input_sender: &mpsc::UnboundedSender<shared::IndexedState<shared::AddBodyEvent>>,
        message: Option<(webrtc_unreliable::MessageType, std::net::SocketAddr)>,
    ) {
        if let Some((message_type, remote_addr)) = message {
            let response = match bincode::deserialize::<shared::Send>(&message_buf) {
                Err(err) => {
                    log::error!("deserialize error: {}", err);
                    None
                }
                Ok(shared::Send::Ping(frame_index)) => Some(shared::Recv::Pong(frame_index)),
                Ok(shared::Send::InputState(input_state)) => {
                    if let Err(err) = input_sender.send(input_state) {
                        log::error!("input send error: {}", err);
                    }
                    broadcast_except(rtc_server, message_buf, remote_addr).await;
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
            message = state_recver.recv() => {
                on_internal_message(&mut rtc_server, message.unwrap()).await;
            },
            message = try_external(&mut rtc_server, &mut message_buf) => {
                on_external_message(&mut rtc_server, &mut message_buf, &input_sender, message).await;
            }
        }
    }
}
