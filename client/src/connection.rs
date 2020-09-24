use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::{cell::RefCell, rc::Rc};

enum Callback {
    OnMessage,
    OnIceCandidate,
}

impl Into<&'static str> for Callback {
    fn into(self) -> &'static str {
        match self {
            Callback::OnMessage => "message",
            Callback::OnIceCandidate => "icecandidate",
        }
    }
}

#[wasm_bindgen]
pub struct Connection {
    peer: web_sys::RtcPeerConnection,
    data_channel: web_sys::RtcDataChannel,
    on_message_callback: Closure<dyn FnMut(web_sys::MessageEvent)>,
    on_ice_candidate_callback: Closure<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>,
    wakers: Rc<RefCell<Vec<std::rc::Weak<RefCell<Option<std::task::Waker>>>>>>,
    receiver: crossbeam_channel::Receiver<Box<[u8]>>,
}

#[wasm_bindgen]
impl Connection {
    #[wasm_bindgen]
    pub async fn connect(peer: web_sys::RtcPeerConnection) -> Result<Connection, JsValue> {
        let data_channel = {
            let mut data_channel_config = web_sys::RtcDataChannelInit::new();
            data_channel_config.ordered(false);
            data_channel_config.max_retransmits(0);
            peer.create_data_channel_with_data_channel_dict("webudp", &data_channel_config)
        };

        let wakers: Rc<RefCell<Vec<std::rc::Weak<RefCell<Option<std::task::Waker>>>>>> =
            Default::default();
        let (sender, receiver) = crossbeam_channel::unbounded();
        let on_message_callback = {
            let wakers = Rc::clone(&wakers);
            let on_message_callback = Closure::wrap(Box::new(move |ev: web_sys::MessageEvent| {
                let data = js_sys::Uint8Array::new(&ev.data());
                let _r = sender.send(data.to_vec().into_boxed_slice());
                let mut wakers = wakers.borrow_mut();
                wakers.retain(|waker| match waker.upgrade() {
                    None => false,
                    Some(waker) => {
                        if let Some(waker) = waker.borrow_mut().take() {
                            waker.wake();
                        }
                        true
                    }
                });
            })
                as Box<dyn FnMut(web_sys::MessageEvent)>);
            data_channel.add_event_listener_with_callback(
                Callback::OnMessage.into(),
                on_message_callback.as_ref().unchecked_ref(),
            )?;
            on_message_callback
        };

        let on_ice_candidate_callback = {
            let on_ice_candidate_callback =
                Closure::wrap(Box::new(move |ev: web_sys::RtcPeerConnectionIceEvent| {
                    match ev.candidate() {
                        Some(candidate) => log::debug!("received ice candidate: {:?}", candidate),
                        None => log::debug!("all local candidates received"),
                    }
                })
                    as Box<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>);
            peer.add_event_listener_with_callback(
                Callback::OnIceCandidate.into(),
                on_ice_candidate_callback.as_ref().unchecked_ref(),
            )?;
            on_ice_candidate_callback
        };

        let data_channel = Rc::new(data_channel);
        let promise = js_sys::Promise::new(&mut move |resolve, reject| {
            {
                let on_open_callback = Closure::wrap(Box::new({
                    let data_channel = Rc::clone(&data_channel);
                    move |_: JsValue| {
                        resolve
                            .call1(&JsValue::undefined(), &data_channel)
                            .unwrap_throw();
                    }
                }) as Box<dyn FnMut(JsValue)>);
                data_channel.set_onopen(Some(on_open_callback.as_ref().unchecked_ref()));
                on_open_callback.forget();
            }

            {
                let on_error_callback = Closure::wrap(Box::new(move |error: JsValue| {
                    reject.call1(&JsValue::undefined(), &error).unwrap_throw();
                })
                    as Box<dyn FnMut(JsValue)>);
                data_channel.set_onerror(Some(on_error_callback.as_ref().unchecked_ref()));
                on_error_callback.forget();
            }
        });

        {
            let offer = wasm_bindgen_futures::JsFuture::from(peer.create_offer()).await?;
            let offer_sdp = js_sys::Reflect::get(&offer, &JsValue::from_str("sdp"))?
                .as_string()
                .unwrap();
            let mut offer_obj = web_sys::RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Offer);
            offer_obj.sdp(&offer_sdp);
            let sld_promise = peer.set_local_description(&offer_obj);
            wasm_bindgen_futures::JsFuture::from(sld_promise).await?;
        }

        {
            let mut opts = web_sys::RequestInit::new();
            opts.method("POST");
            opts.body(Some(&JsValue::from_str(
                &peer.local_description().unwrap().sdp(),
            )));

            let request = web_sys::Request::new_with_str_and_init("/new_rtc_session", &opts)?;

            let window = web_sys::window().unwrap();
            let resp_value =
                wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;

            let resp: web_sys::Response = resp_value.dyn_into()?;
            if !resp.ok() {
                return Err(JsValue::from_str(resp.status_text().as_str()));
            }
            let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;

            let answer = js_sys::Reflect::get(&json, &JsValue::from_str("answer"))?;
            let answer_sdp = js_sys::Reflect::get(&answer, &JsValue::from_str("sdp"))?
                .as_string()
                .unwrap();
            let mut answer_obj =
                web_sys::RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Answer);
            answer_obj.sdp(&answer_sdp);
            wasm_bindgen_futures::JsFuture::from(peer.set_remote_description(&answer_obj)).await?;

            // TODO: validate that this duck-typing into RtcIceCandidate is the best way to do this
            let candidate: web_sys::RtcIceCandidate =
                js_sys::Reflect::get(&json, &JsValue::from_str("candidate"))?.unchecked_into();
            match wasm_bindgen_futures::JsFuture::from(
                peer.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate)),
            )
            .await
            {
                Ok(_) => log::debug!("add ice candidate success"),
                Err(_) => log::error!("add ice candidate failure"),
            }
        }

        let data_channel: web_sys::RtcDataChannel = wasm_bindgen_futures::JsFuture::from(promise)
            .await?
            .dyn_into()?;

        Ok(Self {
            peer,
            data_channel,
            on_message_callback,
            on_ice_candidate_callback,
            receiver,
            wakers,
        })
    }

    #[wasm_bindgen]
    pub fn send_num(&mut self, num: i32) -> Result<(), JsValue> {
        self.send(&num.to_le_bytes())
    }

    #[wasm_bindgen]
    pub fn send_str(&mut self, s: &str) -> Result<(), JsValue> {
        self.send(s.as_bytes())
    }

    #[wasm_bindgen]
    pub fn send(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.data_channel.send_with_u8_array(data)
    }

    #[wasm_bindgen]
    pub fn recv(&mut self) -> Option<Box<[u8]>> {
        self.receiver.try_recv().ok()
    }

    #[wasm_bindgen]
    pub fn recv_fut(&mut self) -> RecvFuture {
        let waker: Rc<RefCell<Option<std::task::Waker>>> = Default::default();
        self.wakers.borrow_mut().push(Rc::downgrade(&waker));
        RecvFuture {
            receiver: self.receiver.clone(),
            waker,
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        log::debug!("Dropping data channel.");

        self.data_channel
            .remove_event_listener_with_callback(
                Callback::OnMessage.into(),
                self.on_message_callback.as_ref().unchecked_ref(),
            )
            .expect("failed to remove message event listener");

        self.peer
            .remove_event_listener_with_callback(
                Callback::OnIceCandidate.into(),
                self.on_ice_candidate_callback.as_ref().unchecked_ref(),
            )
            .expect("failed to remove icecandidate event listener");
    }
}

#[wasm_bindgen]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct RecvFuture {
    receiver: crossbeam_channel::Receiver<Box<[u8]>>,
    waker: Rc<RefCell<Option<std::task::Waker>>>,
}

#[wasm_bindgen]
impl RecvFuture {
    #[wasm_bindgen(js_name = await)]
    pub async fn run(self) -> js_sys::Uint8Array {
        (*self.await).into()
    }
}

impl std::future::Future for RecvFuture {
    type Output = Box<[u8]>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        match self.receiver.try_recv() {
            Ok(frame) => return std::task::Poll::Ready(frame),
            Err(_err) => {
                let waker = cx.waker().clone();
                *self.waker.borrow_mut() = Some(waker);
                std::task::Poll::Pending
            }
        }
    }
}
