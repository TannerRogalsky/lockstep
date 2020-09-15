struct Interval {
    _closure: Closure<dyn FnMut()>,
    handle: i32,
}

impl Interval {
    pub fn new<F: 'static>(timeout: std::time::Duration, f: F) -> Result<Self, JsValue>
        where
            F: FnMut(),
    {
        let closure = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
        let handle = web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                timeout.as_millis() as i32,
            )?;

        Ok(Interval {
            _closure: closure,
            handle,
        })
    }
}

// When the Interval is destroyed, cancel its `setInterval` timer.
impl Drop for Interval {
    fn drop(&mut self) {
        web_sys::window()
            .unwrap()
            .clear_interval_with_handle(self.handle);
    }
}