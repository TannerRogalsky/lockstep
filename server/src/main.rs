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
            http: (localhost, 8080).into(),
            webrtc_data: (localhost, 42424).into(),
            webrtc_public: (localhost, 42424).into(),
            udp: (localhost, 43434).into(),
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let config = AppConfig::default();

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

    let session_endpoint = rtc_server.session_endpoint();
    tokio::spawn(async move {
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
        warp::serve(public.or(rtc)).run(config.http).await;
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
            match rtc_server
                .send(&message_buf, message_type, &remote_addr)
                .await
            {
                Ok(_) => log::debug!("send buf success to {}: {:?}", remote_addr, message_buf),
                Err(err) => log::warn!("could not send message to {}: {}", remote_addr, err),
            }
        }
    }
}
