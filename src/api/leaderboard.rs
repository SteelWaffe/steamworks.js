use napi_derive::napi;

#[napi]
pub mod leaderboard {
    use napi::bindgen_prelude::Error;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    use steamworks::{
        Leaderboard, LeaderboardDataRequest, LeaderboardDisplayType, LeaderboardSortMethod,
        UploadScoreMethod,
    };
    use tokio::sync::oneshot;

    // name -> resolved leaderboard handle (JS only ever passes the name)
    fn handles() -> &'static Mutex<HashMap<String, Leaderboard>> {
        static HANDLES: OnceLock<Mutex<HashMap<String, Leaderboard>>> = OnceLock::new();
        HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    #[napi(object)]
    pub struct UploadResult {
        pub changed: bool,
        pub score: i32,
        pub global_rank_new: u32,
        pub global_rank_previous: u32,
    }

    #[napi(object)]
    pub struct Entry {
        /// 64-bit Steam ID rendered as a decimal string (avoids BigInt over IPC).
        pub steam_id64: String,
        pub global_rank: u32,
        pub score: i32,
    }

    async fn ensure(name: String, descending: bool) -> Result<Leaderboard, Error> {
        if let Some(lb) = handles().lock().unwrap().get(&name).cloned() {
            return Ok(lb);
        }

        let client = crate::client::get_client();
        let sort = if descending {
            LeaderboardSortMethod::Descending
        } else {
            LeaderboardSortMethod::Ascending
        };

        let (tx, rx) = oneshot::channel();
        client.user_stats().find_or_create_leaderboard(
            &name,
            sort,
            LeaderboardDisplayType::Numeric,
            move |result| {
                let _ = tx.send(result);
            },
        );

        let resolved = rx
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?
            .map_err(|e| Error::from_reason(format!("{:?}", e)))?
            .ok_or_else(|| Error::from_reason("Leaderboard not found"))?;

        handles().lock().unwrap().insert(name, resolved.clone());
        Ok(resolved)
    }

    #[napi]
    pub async fn find_or_create_leaderboard(
        name: String,
        descending: bool,
        _numeric: bool,
    ) -> Result<bool, Error> {
        ensure(name, descending).await.map(|_| true)
    }

    #[napi]
    pub async fn upload_leaderboard_score(
        name: String,
        score: i32,
        force: bool,
    ) -> Result<Option<UploadResult>, Error> {
        let client = crate::client::get_client();
        let lb = ensure(name, true).await?;
        let method = if force {
            UploadScoreMethod::ForceUpdate
        } else {
            UploadScoreMethod::KeepBest
        };

        let (tx, rx) = oneshot::channel();
        client
            .user_stats()
            .upload_leaderboard_score(&lb, method, score, &[], move |result| {
                let _ = tx.send(result);
            });

        let result = rx
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?
            .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

        Ok(result.map(|u| UploadResult {
            changed: u.was_changed,
            score: u.score,
            global_rank_new: u.global_rank_new as u32,
            global_rank_previous: u.global_rank_previous as u32,
        }))
    }

    #[napi]
    pub async fn download_leaderboard_entries(
        name: String,
        request: String,
        start: u32,
        end: u32,
    ) -> Result<Vec<Entry>, Error> {
        let client = crate::client::get_client();
        let lb = ensure(name, true).await?;

        let req = match request.as_str() {
            "aroundUser" => LeaderboardDataRequest::GlobalAroundUser,
            "friends" => LeaderboardDataRequest::Friends,
            _ => LeaderboardDataRequest::Global,
        };

        let (tx, rx) = oneshot::channel();
        client.user_stats().download_leaderboard_entries(
            &lb,
            req,
            start as usize,
            end as usize,
            0,
            move |result| {
                let _ = tx.send(result);
            },
        );

        let entries = rx
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?
            .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

        Ok(entries
            .into_iter()
            .map(|e| Entry {
                steam_id64: e.user.raw().to_string(),
                global_rank: e.global_rank as u32,
                score: e.score,
            })
            .collect())
    }

    #[napi]
    pub async fn get_leaderboard_entry_count(name: String) -> Result<u32, Error> {
        let client = crate::client::get_client();
        let lb = ensure(name, true).await?;
        Ok(client.user_stats().get_leaderboard_entry_count(&lb) as u32)
    }
}
