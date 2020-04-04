pub mod config {
    /// Configuration module.
    ///
    /// Gets custom config values from environment variables and the
    /// Rocket.toml config file.  Values set as environment variables will
    /// override like values in the config file.
    use rocket::Rocket;
    use std::env::{var};

    /// The Boss host to talk to.
    pub struct BossHost(pub String);

    const BOSSHOST_ENV_NAME: &str = "BOSSHOST";
    const BOSSHOST_ROCKET_CFG: &str = "bosshost";
    const BOSSHOST_DEFAULT: &str = "api.bossdb.io";

    /// Gets the Boss host to talk to.  First checks for an environment
    /// variable.  Then checks for a value in the Rocket.toml file.
    pub fn get_boss_host(rocket: Rocket) -> Result<Rocket, Rocket> {
        let boss_host: String;
        match var(BOSSHOST_ENV_NAME) {
            Ok(val) => boss_host = val,
            Err(e) => {
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
}

