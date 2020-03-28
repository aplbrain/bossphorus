pub mod intern {
    use ndarray::{s, Array, Array3};
    use reqwest::Client;
    // use reqwest::StatusCode;

    type Extents<'a> = (i32, i32);

    pub struct BossRemote {
        protocol: String,
        host: String,
        token: String,
        client: Client,
    }

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
            format!("{}://{}/{}/v1/", self.protocol, self.host, suffix)
        }

        pub fn get_cutout(
            &self,
            boss_uri: String,
            res: u8,
            xs: Extents,
            ys: Extents,
            zs: Extents,
        ) -> Array3<u8> {
            let (col, exp, chan) = parse_bossdb_uri(boss_uri);
            let url = self.build_url(format!(
                "cutout/{col}/{exp}/{chan}/{res}/{xs_start}:{xs_stop}/{ys_start}:{ys_stop}/{zs_start}:{zs_stop}",
                col=col, exp=exp, chan=chan, res=res,
                xs_start = xs.0, xs_stop = xs.1,
                ys_start = ys.0, ys_stop = ys.1,
                zs_start = zs.0, zs_stop = zs.1,
            ));
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("token {}", self.token))
                .send();
            // match response.status() {
            //     StatusCode::OK => response.text(),
            //     err => panic!(err.to_string()),
            // }

            // return Array3.from(vec![]);
        }
    }
}
