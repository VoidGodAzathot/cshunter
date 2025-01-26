pub struct NetId;

impl NetId {
    pub async fn get_ip_addr() -> Option<String> {
        match reqwest::get("https://api.ipify.org/?format=raw").await {
            Ok(body) => {
                match body.text().await {
                    Ok(raw) => {
                        return Some(raw);
                    },

                    Err(e) => {
                        if cfg!(dev) {
                            println!("{e:?}")
                        }
                    }
                }
            },
            
            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}")
                }
            }
        }

        None
    }
}
