
#[cfg(not(target_family = "wasm"))]
pub use iced_winit::Clipboard;


#[cfg(target_family = "wasm")]
#[cfg(web_sys_unstable_apis)]
mod web_clipboard {

    use std::{rc::Rc, cell::RefCell};
    use crate::{EventExt, EventLoop, EventLoopProxy};
    use web_sys::{Clipboard as WebClipboard, ClipboardEvent};
    use js_sys::Function;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen::{JsValue, closure::Closure};


    // helper

    struct ClipboardHandle { clipboard: WebClipboard, read: bool, write: bool }

    impl ClipboardHandle {
        fn new() -> Option<Self> {
            (|| {
                let clipboard = web_sys::window().and_then(|win| win.navigator().clipboard())
                    .ok_or("navigator.clipboard is not available")?;

                let read = JsValue::from("readText").js_in(&clipboard);
                let write = JsValue::from("writeText").js_in(&clipboard);

                Ok(Self {clipboard, read, write})
            })
            ().map_err(|err: &str| log::warn!("{err}")).ok()
        }
    }


    struct PasteListener { listener: Function }

    impl PasteListener {

        fn new(clipboard_content: Rc<RefCell<Option<String>>>, event_proxy: Option<EventLoopProxy>) -> Self {

            let closure: Box<dyn Fn(ClipboardEvent)> = if let Some(event_proxy) = event_proxy {
                Box::new(move |evt| {
                    if let Some(transfer) = evt.clipboard_data() {
                        clipboard_content.replace(
                            transfer.get_data("text")
                            .map_err(|err| log::error!("{:?}", err)).ok()
                        );
                    }
                    if let Err(err) = event_proxy.send_event(EventExt::ClipboardPaste) {
                        log::error!("{:?}", err);
                    }
                })
            }
            else {
                Box::new(move |evt| {
                    if let Some(transfer) = evt.clipboard_data() {
                        clipboard_content.replace(
                            transfer.get_data("text")
                            .map_err(|err| log::error!("{:?}", err)).ok()
                        );
                    }
                })
            };

            Self { listener: Closure::wrap(closure).into_js_value().into() }
        }

        fn attach(&self) -> Result<(), &'static str> {
            let document = web_sys::window().and_then(|win| win.document()).ok_or("couldn't get window.document")?;

            document.add_event_listener_with_callback("paste", &self.listener)
                .or(Err("couldn't attach PasteListener to document"))?;

            Ok(())
        }

        fn detach(&self) {
            (|| {
                let document = web_sys::window().and_then(|win| win.document()).ok_or("couldn't get window.document")?;

                document.remove_event_listener_with_callback("paste", &self.listener)
                    .or(Err("couldn't detach PasteListener to document"))?;

                Ok(())
            })
            ().unwrap_or_else(|err: &str| log::error!("{err}"));
        }
    }


    // main clipboard

    pub struct Clipboard {
        content: Rc<RefCell<Option<String>>>,
        handle: Option<ClipboardHandle>,
        paste_listener: Option<PasteListener>,
        event_proxy: Option<Rc<EventLoopProxy>>,
    }

    impl Clipboard {

        pub fn connect(event_loop: &EventLoop) -> Self {

            let content = RefCell::new(None).into();
            let paste_listener = PasteListener::new(Rc::clone(&content), Some(event_loop.create_proxy()));

            Self {
                content,
                handle: ClipboardHandle::new(),
                event_proxy: Some(event_loop.create_proxy().into()),
                paste_listener: match paste_listener.attach() {
                    Ok(()) => Some(paste_listener),
                    Err(err) => {
                        log::error!("{err}");
                        None
                    }
                }
            }
        }

        pub fn unconnected() -> Self {

            let content = RefCell::new(None).into();
            let paste_listener = PasteListener::new(Rc::clone(&content), None);

            Self {
                content,
                handle: ClipboardHandle::new(),
                event_proxy: None,
                paste_listener: match paste_listener.attach() {
                    Ok(()) => Some(paste_listener),
                    Err(err) => {
                        log::error!("{err}");
                        None
                    }
                }
            }
        }

        pub fn fetch(&self) { // fetches content from system clipboard asynchronously
            if let Some(ClipboardHandle {read: true, clipboard, ..}) = &self.handle {

                let content = Rc::clone(&self.content);
                let promise = clipboard.read_text();

                if let Some(event_proxy) = &self.event_proxy {
                    let event_proxy = Rc::clone(event_proxy);

                    wasm_bindgen_futures::spawn_local(async move {
                        content.replace(
                            match JsFuture::from(promise).await {
                                Ok(res) => res.as_string(),
                                Err(err) => { log::error!("{:?}", err); None },
                            }
                        );
                        if let Err(err) = event_proxy.send_event(EventExt::ClipboardFetch) {
                            log::error!("{:?}", err);
                        }
                    });
                }
                else {
                    wasm_bindgen_futures::spawn_local(async move {
                        content.replace(
                            match JsFuture::from(promise).await {
                                Ok(res) => res.as_string(),
                                Err(err) => { log::error!("{:?}", err); None },
                            }
                        );
                    });
                }
            }
        }

        pub fn read(&self) -> Option<String> {
            self.content.borrow().as_ref().cloned()
        }

        pub fn write(&mut self, text: String) {
            if let Some(ClipboardHandle {write: true, clipboard, ..}) = &self.handle {
                let _promise = clipboard.write_text(&text);
            }
            self.content.replace(Some(text));
        }


        // introspective methods

        pub fn is_connected(&self) -> bool {
            self.event_proxy.is_some()
        }

        pub fn is_listening(&self) -> bool {
            self.paste_listener.is_some()
        }

        pub fn can_write(&self) -> bool {
            matches!(self.handle, Some(ClipboardHandle {write: true, ..}))
        }

        pub fn can_fetch(&self) -> bool {
            matches!(self.handle, Some(ClipboardHandle {read: true, ..}))
        }
    }


    impl Drop for Clipboard {
        fn drop(&mut self) {
            if let Some(listener) = &self.paste_listener {
                listener.detach()
            }
        }
    }


    // make usable with iced
    impl iced_native::clipboard::Clipboard for Clipboard {
        fn read(&self) -> Option<String> { Clipboard::read(self) }
        fn write(&mut self, text: String) { Clipboard::write(self, text) }
    }
}


#[cfg(target_family = "wasm")]
pub use web_clipboard::Clipboard;
