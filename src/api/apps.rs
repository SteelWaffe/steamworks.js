use super::localplayer::PlayerSteamId;
use napi_derive::napi;

#[napi]
pub mod apps {
    use super::PlayerSteamId;
    use steamworks::AppId;
    use steamworks_sys as sys;

    #[napi]
    pub fn is_subscribed_app(app_id: u32) -> bool {
        let client = crate::client::get_client();
        client.apps().is_subscribed_app(AppId(app_id))
    }
    #[napi]
    pub fn is_app_installed(app_id: u32) -> bool {
        let client = crate::client::get_client();
        client.apps().is_app_installed(AppId(app_id))
    }

    #[napi]
    pub fn is_dlc_installed(app_id: u32) -> bool {
        let client = crate::client::get_client();
        client.apps().is_dlc_installed(AppId(app_id))
    }

    /// Asks Steam to download and install a DLC the user owns but has switched
    /// off or not downloaded yet (`ISteamApps::InstallDLC`). Steam does the work
    /// in the background; `is_dlc_installed` turns true once it is on disk.
    #[napi]
    pub fn install_dlc(app_id: u32) {
        // The handle is only valid once the client is up; asking for it first
        // keeps this from running before init like every other call here.
        let _client = crate::client::get_client();
        unsafe { sys::SteamAPI_ISteamApps_InstallDLC(sys::SteamAPI_SteamApps_v008(), app_id) }
    }

    #[napi(object)]
    pub struct DlcDownloadProgress {
        pub bytes_downloaded: f64,
        pub bytes_total: f64,
    }

    /// How far a DLC download has come, or null when Steam is not downloading it
    /// (`ISteamApps::GetDlcDownloadProgress`).
    #[napi]
    pub fn get_dlc_download_progress(app_id: u32) -> Option<DlcDownloadProgress> {
        let _client = crate::client::get_client();
        let mut downloaded: u64 = 0;
        let mut total: u64 = 0;
        let active = unsafe {
            sys::SteamAPI_ISteamApps_GetDlcDownloadProgress(
                sys::SteamAPI_SteamApps_v008(),
                app_id,
                &mut downloaded,
                &mut total,
            )
        };
        active.then(|| DlcDownloadProgress {
            bytes_downloaded: downloaded as f64,
            bytes_total: total as f64,
        })
    }

    #[napi]
    pub fn is_subscribed_from_free_weekend() -> bool {
        let client = crate::client::get_client();
        client.apps().is_subscribed_from_free_weekend()
    }

    #[napi]
    pub fn is_vac_banned() -> bool {
        let client = crate::client::get_client();
        client.apps().is_vac_banned()
    }

    #[napi]
    pub fn is_cybercafe() -> bool {
        let client = crate::client::get_client();
        client.apps().is_cybercafe()
    }

    #[napi]
    pub fn is_low_violence() -> bool {
        let client = crate::client::get_client();
        client.apps().is_low_violence()
    }

    #[napi]
    pub fn is_subscribed() -> bool {
        let client = crate::client::get_client();
        client.apps().is_subscribed()
    }

    #[napi]
    pub fn app_build_id() -> i32 {
        let client = crate::client::get_client();
        client.apps().app_build_id()
    }

    #[napi]
    pub fn app_install_dir(app_id: u32) -> String {
        let client = crate::client::get_client();
        client.apps().app_install_dir(AppId(app_id))
    }

    #[napi]
    pub fn app_owner() -> PlayerSteamId {
        let client = crate::client::get_client();
        let steam_id = client.apps().app_owner();
        PlayerSteamId::from_steamid(steam_id)
    }

    #[napi]
    pub fn available_game_languages() -> Vec<String> {
        let client = crate::client::get_client();
        client.apps().available_game_languages()
    }

    #[napi]
    pub fn current_game_language() -> String {
        let client = crate::client::get_client();
        client.apps().current_game_language()
    }

    #[napi]
    pub fn current_beta_name() -> Option<String> {
        let client = crate::client::get_client();
        client.apps().current_beta_name()
    }
}
