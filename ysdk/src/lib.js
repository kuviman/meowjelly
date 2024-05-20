// Informing the SDK that the game has loaded and is ready to play
export function ready(ysdk) {
    ysdk.features.LoadingAPI?.ready();
}

export function show_fullscreen_adv(ysdk, onClose, onOpen, onError, onOffline) {
    ysdk.adv.showFullscreenAdv({
        callbacks: {
            onClose,
            onOpen,
            onError,
            onOffline,
        },
    });
}