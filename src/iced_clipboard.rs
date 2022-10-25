
#[cfg(not(target_family = "wasm"))]
pub use iced_winit::Clipboard;


#[cfg(target_family = "wasm")]
#[cfg(web_sys_unstable_apis)]
mod web_clipboard {

    use std::{rc::Rc, cell::RefCell};
    use crate::{winit::window::Window};
    use web_sys::{Clipboard as WebClipboard};
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen::JsValue;


    pub struct Clipboard {
        clipboard: Option<Box<WebClipboard>>,
        content: Rc<RefCell<String>>,
        read: bool,
        write: bool,
    }

    impl Clipboard {

        pub fn connect(_window: &Window) -> Self { Self::unconnected() } // mimic iced_winit Clipboard

        pub fn unconnected() -> Self {

            let (mut read, mut write) = (false, false);

            let clipboard = match (|| {
                let window = web_sys::window().ok_or("couldn't get web_sys::Window")?;

                let clipboard = window.navigator().clipboard().ok_or("navigator.clipboard is not available")?;

                read = JsValue::from("readText").js_in(&clipboard);
                write = JsValue::from("writeText").js_in(&clipboard);

                Ok::<web_sys::Clipboard, String>(clipboard)
            })() {
                Ok(clipboard) => Some(Box::new(clipboard)),
                Err(err) => { log::warn!("{err}"); None },
            };

            Self { clipboard, content: RefCell::new("".to_string()).into(), read, write }
        }

        pub fn fetch(&self) {
            if let (true, Some(clipboard)) = (self.read, &self.clipboard) {

                let content = Rc::clone(&self.content);
                let promise = clipboard.read_text();

                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(res) = JsFuture::from(promise).await {
                        if let Some(text) = res.as_string() {
                            content.replace(text);
                        }
                    }
                });
            }
        }

        pub fn read(&self) -> Option<String> {
            let content = self.content.borrow();
            if !content.is_empty() { Some(content.to_string()) }
            else { None }
        }

        pub fn write(&mut self, text: String) {
            if let (true, Some(clipboard)) = (self.write, &self.clipboard) {
                let _promise = clipboard.write_text(&text);
            }
            *self.content.borrow_mut() = text;
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
