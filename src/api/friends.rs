use napi_derive::napi;
use serde::Serialize;
use std::ffi::{c_void, CStr};
use steamworks_sys as sys;

/// `GameRichPresenceJoinRequested_t`: the player accepted a game invite (or
/// "Join game") from the Steam friends list while the game was already running.
/// steamworks-rs does not bind it, so it is declared here.
#[derive(Clone, Debug, Serialize)]
pub struct GameRichPresenceJoinRequested {
    /// 64-bit Steam ID as a decimal string (a JSON number would lose precision in JS).
    pub friend_steam_id: String,
    pub connect: String,
}

unsafe impl steamworks::Callback for GameRichPresenceJoinRequested {
    // k_iSteamFriendsCallbacks (300) + 37
    const ID: i32 = 337;
    const SIZE: i32 = std::mem::size_of::<sys::GameRichPresenceJoinRequested_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &*(raw as *const sys::GameRichPresenceJoinRequested_t);
        let connect = CStr::from_ptr(val.m_rgchConnect.as_ptr())
            .to_string_lossy()
            .into_owned();
        // The SDK struct is packed, so the id is read unaligned.
        let friend =
            std::ptr::addr_of!(val.m_steamIDFriend.m_steamid.m_unAll64Bits).read_unaligned();
        Self {
            friend_steam_id: friend.to_string(),
            connect,
        }
    }
}

#[napi]
pub mod friends {
    use napi::bindgen_prelude::{Buffer, Error};
    use steamworks::{FriendFlags, FriendState, SteamId};

    #[napi(object)]
    pub struct FriendInfo {
        /// 64-bit Steam ID as a decimal string.
        pub steam_id64: String,
        pub name: String,
        pub nick_name: Option<String>,
        /// `offline`, `online`, `busy`, `away`, `snooze`, `trade` or `play`.
        pub state: String,
        /// App the friend is playing right now, if any.
        pub app_id: Option<u32>,
    }

    #[napi(object)]
    pub struct Avatar {
        pub width: u32,
        pub height: u32,
        /// Raw RGBA pixels.
        pub rgba: Buffer,
    }

    fn parse_id(steam_id64: &str) -> Result<SteamId, Error> {
        steam_id64
            .parse::<u64>()
            .map(SteamId::from_raw)
            .map_err(|_| Error::from_reason("invalid steam id"))
    }

    fn state_name(state: FriendState) -> &'static str {
        match state {
            FriendState::Offline => "offline",
            FriendState::Online => "online",
            FriendState::Busy => "busy",
            FriendState::Away => "away",
            FriendState::Snooze => "snooze",
            FriendState::LookingToTrade => "trade",
            FriendState::LookingToPlay => "play",
        }
    }

    /// The player's regular Steam friends (not blocked, pending or clan members).
    #[napi]
    pub fn get_friends() -> Vec<FriendInfo> {
        let client = crate::client::get_client();
        client
            .friends()
            .get_friends(FriendFlags::IMMEDIATE)
            .into_iter()
            .map(|f| FriendInfo {
                steam_id64: f.id().raw().to_string(),
                name: f.name(),
                nick_name: f.nick_name(),
                state: state_name(f.state()).to_string(),
                app_id: f.game_played().map(|g| g.game.app_id().0),
            })
            .collect()
    }

    /// Medium (64x64) avatar of any user Steam knows about, or null while it is not downloaded yet.
    #[napi]
    pub fn get_avatar(steam_id64: String) -> Result<Option<Avatar>, Error> {
        let id = parse_id(&steam_id64)?;
        let client = crate::client::get_client();
        Ok(client
            .friends()
            .get_friend(id)
            .medium_avatar()
            .map(|rgba| Avatar {
                width: 64,
                height: 64,
                rgba: rgba.into(),
            }))
    }

    /// Asks Steam to download a user's name and avatar; a PersonaStateChange follows.
    /// False when the data is already cached.
    #[napi]
    pub fn request_user_information(steam_id64: String, name_only: bool) -> Result<bool, Error> {
        let id = parse_id(&steam_id64)?;
        let client = crate::client::get_client();
        Ok(client.friends().request_user_information(id, name_only))
    }

    /// Sends a Steam game invite. The friend's game starts with `connect` on its
    /// command line, or receives GameRichPresenceJoinRequested if already running.
    #[napi]
    pub fn invite_user_to_game(steam_id64: String, connect: String) -> Result<(), Error> {
        let id = parse_id(&steam_id64)?;
        let client = crate::client::get_client();
        client
            .friends()
            .get_friend(id)
            .invite_user_to_game(&connect);
        Ok(())
    }
}
