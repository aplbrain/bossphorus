/// Configuration module.
///
/// Gets custom config values from environment variables and the
/// Rocket.toml config file.  Values set as environment variables will
/// override like values in the config file.
use rocket::Rocket;
use std::env;
use std::fs;

/// Store cuboid files off of this folder.  This is not a standard config
/// variable because we will likely move to a separate config file as
/// discussed in https://github.com/aplbrain/bossphorust/issues/11
/// Thus, keeping this as a simple constant for now.
pub const CUBOID_ROOT_PATH: &str = "uploads";

/// Get the absolute path of the cuboid root folder.
pub fn get_cuboid_root_abs_path() -> String {
    let path = fs::canonicalize(CUBOID_ROOT_PATH);
    let path_str = match path {
        Ok(p) => p,
        Err(_) => {
            fs::create_dir_all(CUBOID_ROOT_PATH)
                .expect(&format!("Couldn't create {}", CUBOID_ROOT_PATH));
            return get_cuboid_root_abs_path()
        },
    };
    return match path_str.as_path().to_str() {
        Some(s) => s.to_string(),
        None => panic!("Non-unicode path given"),
    }
}

/// The Boss host to talk to.
pub struct BossHost(pub String);

const BOSSHOST_ENV_NAME: &str = "BOSSHOST";
const BOSSHOST_ROCKET_CFG: &str = "bosshost";
const BOSSHOST_DEFAULT: &str = "api.bossdb.io";

/// Gets the Boss host to talk to.  First checks for an environment
/// variable.  Then checks for a value in the Rocket.toml file.
pub fn get_boss_host(rocket: Rocket) -> Result<Rocket, Rocket> {
    let boss_host: String;
    match env::var(BOSSHOST_ENV_NAME) {
        Ok(val) => boss_host = val,
        Err(_) => {
            boss_host = rocket.config()
                .get_str(BOSSHOST_ROCKET_CFG)
                .unwrap_or(BOSSHOST_DEFAULT)
                .to_string();
        },
    }
    // ToDo: make this visible to the user in a better place.
    println!("Boss host: {}", boss_host);
    Ok(rocket.manage(BossHost(boss_host)))
}


/// Boss token used for auth.
pub struct BossToken(pub String);

const BOSSTOKEN_ENV_NAME: &str = "BOSSTOKEN";
const BOSSTOKEN_ROCKET_CFG: &str = "bosstoken";
const BOSSTOKEN_DEFAULT: &str = "public";

/// Gets the Boss token for auth.  First checks for an environment
/// variable.  Then checks for a value in the Rocket.toml file.
pub fn get_boss_token(rocket: Rocket) -> Result<Rocket, Rocket> {
    let boss_token: String;
    match env::var(BOSSTOKEN_ENV_NAME) {
        Ok(val) => boss_token = val,
        Err(_) => {
            boss_token = rocket.config()
                .get_str(BOSSTOKEN_ROCKET_CFG)
                .unwrap_or(BOSSTOKEN_DEFAULT)
                .to_string();
        },
    }
    Ok(rocket.manage(BossToken(boss_token)))
}

/// Boss token used for auth.
pub struct UsageTracker(pub String);

const USAGE_MGR_ENV_NAME: &str = "USAGE_MGR";
const USAGE_MGR_ROCKET_CFG: &str = "usage_mgr";
const USAGE_MGR_DEFAULT: &str = "none";

/// Gets the usage manager to use.  First checks for an environment
/// variable.  Then checks for a value in the Rocket.toml file.
pub fn get_usage_mgr(rocket: Rocket) -> Result<Rocket, Rocket> {
    let usage_mgr: String;
    match env::var(USAGE_MGR_ENV_NAME) {
        Ok(val) => usage_mgr = val,
        Err(_) => {
            usage_mgr = rocket.config()
                .get_str(USAGE_MGR_ROCKET_CFG)
                .unwrap_or(USAGE_MGR_DEFAULT)
                .to_string();
        },
    }
    Ok(rocket.manage(UsageTracker(usage_mgr)))
}
