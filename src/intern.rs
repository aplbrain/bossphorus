pub mod remote {
    /// This module is intended to begin to mirror the intern Python library.
    use ndarray::{s, Array, Array3};
    use reqwest::blocking::Client;

    pub struct BossRemote {
        /// A BossRemote analog to Python's `intern.remote.boss.BossRemote`.
        protocol: String,
        host: String,
        token: String,
        client: Client,
    }

    /// Parse a URI and return a collection, experiment, and channel.
    ///
    /// # Arguments
    ///
    /// * `boss_uri` - A URI like `bossdb://col/exp/chan`
    ///
    /// # Returns
    ///
    /// * Collection, experiment, and channel strings
    ///
    fn parse_bossdb_uri(boss_uri: String) -> (String, String, String) {
        let boss_components = boss_uri.split("://").collect::<Vec<&str>>()[1].to_string();
        let col_exp_chan = boss_components.split("/").collect::<Vec<&str>>();
        return (
            col_exp_chan[0].to_string(),
            col_exp_chan[1].to_string(),
            col_exp_chan[2].to_string(),
        );
    }

    impl BossRemote {
        /// A BossRemote handles its own authentication, etc.
        ///
        /// # Arguments
        ///
        /// * `protocol` - e.g. "https"
        /// * `host` - e.g. "bossdb.io"
        /// * `token` - e.g. "public"
        ///
        pub fn new(protocol: String, host: String, token: String) -> BossRemote {
            let br = BossRemote {
                protocol,
                host,
                token,
                client: Client::new(),
            };
            return br;
        }

        fn build_url(&self, suffix: String) -> String {
            format!("{}://{}/v1/{}/", self.protocol, self.host, suffix)
        }

        /// Get a cutout from the bosslike remote.
        ///
        /// # Arguments
        ///
        /// * `boss_uri` - String
        /// * `res` - u8
        /// * `xs` - Extents
        /// * `ys` - Extents
        /// * `zs` - Extents
        ///
        /// # Returns
        ///
        /// * Array3
        ///
        pub fn get_cutout(
            &self,
            boss_uri: String,
            res: u8,
            xs: (u64, u64),
            ys: (u64, u64),
            zs: (u64, u64),
        ) -> Result<Array3<u8>, reqwest::Error> {
            let (col, exp, chan) = parse_bossdb_uri(boss_uri);
            let url = self.build_url(format!(
                "cutout/{col}/{exp}/{chan}/{res}/{xs_start}:{xs_stop}/{ys_start}:{ys_stop}/{zs_start}:{zs_stop}",
                col=col, exp=exp, chan=chan, res=res,
                xs_start = xs.0, xs_stop = xs.1,
                ys_start = ys.0, ys_stop = ys.1,
                zs_start = zs.0, zs_stop = zs.1,
            ));
            println!("{}", url);
            let mut resp = self
                .client
                .get(&url)
                .header("Authorization", format!("token {}", self.token))
                .send()?;
            if resp.status().is_success() {
                let mut buf = Vec::new();
                std::io::copy(&mut resp, &mut buf).unwrap();
                // decompress:
                let decompressed: Vec<u8> = match unsafe { blosc::decompress_bytes(&buf[..]) } {
                    Ok(a) => a,
                    _ => unreachable!(),
                };
                return Ok(Array::from_shape_vec(
                    (
                        (zs.1 - zs.0) as usize,
                        (ys.1 - ys.0) as usize,
                        (xs.1 - xs.0) as usize,
                    ),
                    decompressed,
                )
                .unwrap());
            } else {
                panic!(format!("{}: {:?}", url, resp.status()))
            }
        }
    }
}
