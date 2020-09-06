use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub struct Connection {
    peer: web_sys::RtcPeerConnection,
    data_channel: web_sys::RtcDataChannel,
    on_message_callback: Closure<dyn FnMut(web_sys::MessageEvent)>,
    on_open_callback: Closure<dyn FnMut(JsValue)>,
    on_ice_candidate_callback: Closure<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>,

    connected: Rc<Cell<bool>>,
    send_buffer: Vec<Vec<u8>>,
    recv_buffer: Rc<RefCell<std::collections::VecDeque<Vec<u8>>>>,
}

#[wasm_bindgen]
impl Connection {
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
        if self.connected.get() {
            for frame in self.send_buffer.drain(..) {
                self.data_channel.send_with_u8_array(&frame)?;
            }
            self.data_channel.send_with_u8_array(data)
        } else {
            self.send_buffer.push(data.to_vec());
            Ok(())
        }
    }

    #[wasm_bindgen]
    pub fn recv(&mut self) -> Option<Box<[u8]>> {
        self.recv_buffer
            .borrow_mut()
            .pop_front()
            .map(Vec::into_boxed_slice)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        log::debug!("Dropping data channel.");

        self.data_channel
            .remove_event_listener_with_callback(
                "message",
                self.on_message_callback.as_ref().unchecked_ref(),
            )
            .expect("failed to remove message event listener");
        self.data_channel
            .remove_event_listener_with_callback(
                "open",
                self.on_open_callback.as_ref().unchecked_ref(),
            )
            .expect("failed to remove open event listener");

        self.peer
            .remove_event_listener_with_callback(
                "icecandidate",
                self.on_ice_candidate_callback.as_ref().unchecked_ref(),
            )
            .expect("failed to remove icecandidate event listener");
    }
}

#[wasm_bindgen]
pub async fn connect(peer: web_sys::RtcPeerConnection) -> Result<Connection, JsValue> {
    let data_channel = {
        let mut data_channel_config = web_sys::RtcDataChannelInit::new();
        data_channel_config.ordered(false);
        data_channel_config.max_retransmits(0);
        peer.create_data_channel_with_data_channel_dict("webudp", &data_channel_config)
    };

    let recv_buffer = Rc::new(RefCell::new(std::collections::VecDeque::new()));

    let on_message_callback = {
        let recv_buffer = Rc::clone(&recv_buffer);
        let on_message_callback = Closure::wrap(Box::new(move |ev: web_sys::MessageEvent| {
            let data = js_sys::Uint8Array::new(&ev.data());
            recv_buffer.borrow_mut().push_back(data.to_vec());
        })
            as Box<dyn FnMut(web_sys::MessageEvent)>);
        data_channel.set_onmessage(Some(on_message_callback.as_ref().unchecked_ref()));
        on_message_callback
    };

    let connected = Rc::new(Cell::new(false));
    let on_open_callback = {
        let connected = Rc::clone(&connected);
        let on_open_callback = Closure::wrap(Box::new(move |_: JsValue| {
            connected.set(true);
        }) as Box<dyn FnMut(JsValue)>);
        data_channel.set_onopen(Some(on_open_callback.as_ref().unchecked_ref()));
        on_open_callback
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
        peer.set_onicecandidate(Some(on_ice_candidate_callback.as_ref().unchecked_ref()));
        on_ice_candidate_callback
    };

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
        let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;

        let answer = js_sys::Reflect::get(&json, &JsValue::from_str("answer"))?;
        let answer_sdp = js_sys::Reflect::get(&answer, &JsValue::from_str("sdp"))?
            .as_string()
            .unwrap();
        let mut answer_obj = web_sys::RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Answer);
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

    Ok(Connection {
        peer,
        data_channel,
        on_message_callback,
        on_open_callback,
        on_ice_candidate_callback,
        connected,
        send_buffer: Vec::new(),
        recv_buffer,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
