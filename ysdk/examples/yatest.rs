fn main() {
    batbox_logger::init();
    wasm_bindgen_futures::spawn_local(async {
        let sdk = ysdk::Ysdk::init().await.unwrap();
        log::info!("YO");
        log::info!("mobile = {:?}", sdk.device_info().is_mobile());
        log::info!("desktop = {:?}", sdk.device_info().is_desktop());
        log::info!("tablet = {:?}", sdk.device_info().is_tablet());
        sdk.ready();
    });
}
