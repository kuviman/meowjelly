use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;

mod ya_sys {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        pub type YaGames;

        #[wasm_bindgen(js_namespace = YaGames)]
        pub fn init() -> js_sys::Promise;

        pub type DeviceInfo;

        #[wasm_bindgen(method, getter)]
        pub fn deviceInfo(this: &YaGames) -> DeviceInfo;

        #[wasm_bindgen(method)]
        pub fn isMobile(this: &DeviceInfo) -> bool;
        #[wasm_bindgen(method)]
        pub fn isDesktop(this: &DeviceInfo) -> bool;
        #[wasm_bindgen(method)]
        pub fn isTablet(this: &DeviceInfo) -> bool;
    }

    #[wasm_bindgen(module = "/src/lib.js")]
    extern "C" {
        pub fn ready(ysdk: &YaGames);
        pub fn show_fullscreen_adv(
            ysdk: &YaGames,
            on_close: Option<js_sys::Function>,
            on_open: Option<js_sys::Function>,
            on_error: Option<js_sys::Function>,
            on_offline: Option<js_sys::Function>,
        );
    }
}

pub struct Ysdk {
    inner: ya_sys::YaGames,
}

impl Ysdk {
    /// Informing the SDK that the game has loaded and is ready to play
    pub fn ready(&self) {
        ya_sys::ready(&self.inner);
    }

    pub async fn show_fullscreen_adv(&self) -> Result<bool, Error> {
        let (sender, receiver) = async_oneshot::oneshot();
        let sender = Rc::new(RefCell::new(sender));
        let on_close: js_sys::Function = wasm_bindgen::closure::Closure::once_into_js({
            let sender = sender.clone();
            move |was_shown: bool| {
                let _ = sender.borrow_mut().send(Ok(was_shown));
            }
        })
        .dyn_into()
        .unwrap();
        let on_error: js_sys::Function = wasm_bindgen::closure::Closure::once_into_js({
            let sender = sender.clone();
            move |error: JsValue| {
                let _ = sender.borrow_mut().send(Err(Error::Js(error)));
            }
        })
        .dyn_into()
        .unwrap();
        let on_offline: js_sys::Function = wasm_bindgen::closure::Closure::once_into_js({
            let sender = sender.clone();
            move || {
                let _ = sender.borrow_mut().send(Err(Error::Offline));
            }
        })
        .dyn_into()
        .unwrap();
        ya_sys::show_fullscreen_adv(
            &self.inner,
            Some(on_close),
            None,
            Some(on_error),
            Some(on_offline),
        );
        receiver.await.unwrap()
    }
}

pub struct DeviceInfo(ya_sys::DeviceInfo);

impl Ysdk {
    pub fn device_info(&self) -> DeviceInfo {
        DeviceInfo(self.inner.deviceInfo())
    }
}

impl DeviceInfo {
    pub fn is_mobile(&self) -> bool {
        self.0.isMobile()
    }
    pub fn is_desktop(&self) -> bool {
        self.0.isDesktop()
    }
    pub fn is_tablet(&self) -> bool {
        self.0.isTablet()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("JavaScript exception: {0:?}")]
    Js(JsValue),
    #[error("offline")]
    Offline,
    #[error("unknown")]
    Unknown,
}

impl Ysdk {
    pub async fn init() -> Result<Self, Error> {
        let document = web_sys::window().unwrap().document().unwrap();
        let existing_script = document
            .get_elements_by_tag_name("script")
            .get_with_index(0)
            .unwrap();
        let new_script: web_sys::HtmlScriptElement = document
            .create_element("script")
            .unwrap()
            .dyn_into()
            .unwrap();
        new_script.set_src("https://yandex.ru/games/sdk/v2");
        new_script.set_async(true);
        existing_script
            .parent_node()
            .unwrap()
            .insert_before(&new_script, Some(&existing_script))
            .unwrap();

        let (mut sender, receiver) = async_oneshot::oneshot();
        let on_load =
            wasm_bindgen::closure::Closure::once_into_js(move |_event: web_sys::Event| {
                let _ = sender.send(());
            });
        new_script.set_onload(Some(&on_load.dyn_into().unwrap()));
        let () = receiver
            .await
            .map_err(|async_oneshot::Closed()| Error::Unknown)?;
        Ok(Self {
            inner: {
                let obj = wasm_bindgen_futures::JsFuture::from(ya_sys::YaGames::init())
                    .await
                    .unwrap();
                obj.unchecked_into()
            },
        })
    }
}
