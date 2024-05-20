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

export function get_player(ysdk, scopes) {
    return ysdk.getPlayer({ scopes })
}

export function player_unique_id(player) {
    return player.getUniqueID()
}

export async function get_player_numeric_data(player, key) {
    let stats = await player.getStats([key]);
    return stats[key];
}

export async function set_player_numeric_data(player, key, value) {
    return await player.setStats({ [key]: value });
}