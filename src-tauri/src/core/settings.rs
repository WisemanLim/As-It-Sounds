use super::db::Db;
use super::error::AppResult;
use super::models::Settings;

const KEY: &str = "settings";

pub fn load(db: &Db) -> AppResult<Settings> {
    if let Some(raw) = db.get_setting(KEY)? {
        match serde_json::from_str::<Settings>(&raw) {
            Ok(s) => Ok(s),
            Err(_) => Ok(Settings::default()),
        }
    } else {
        Ok(Settings::default())
    }
}

pub fn save(db: &Db, s: &Settings) -> AppResult<()> {
    let raw = serde_json::to_string(s)?;
    db.set_setting(KEY, &raw)
}
