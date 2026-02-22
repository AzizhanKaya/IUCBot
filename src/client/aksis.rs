use anyhow::anyhow;
use anyhow::{Result, bail};
use reqwest::{
    Url,
    blocking::Client,
    cookie::{CookieStore, Jar},
    header::{self, HeaderMap, HeaderValue},
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;

const CACHE_FILE: &str = ".aksis.cache";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Cache {
    pub username: Option<String>,
    pub password: Option<String>,
    pub aksis_cookie: Option<String>,
}

impl Cache {
    pub fn load() -> Self {
        if let Ok(data) = fs::read_to_string(CACHE_FILE) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Cache::default()
        }
    }

    pub fn save(&self) {
        fs::write(CACHE_FILE, serde_json::to_string_pretty(self).unwrap()).unwrap();
    }

    pub fn clear(&self) {
        fs::write(CACHE_FILE, "{}").unwrap();
    }
}

pub struct AksisClient {
    client: Client,
    pub cache: Cache,
    pub jar: Arc<Jar>,
}

impl AksisClient {
    pub fn new() -> Self {
        let cache = Cache::load();
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .cookie_store(true)
            .cookie_provider(jar.clone())
            .build()
            .unwrap();

        AksisClient { client, cache, jar }
    }

    fn get_csrf(&self, html: &str) -> Result<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse(r#"input[name="__RequestVerificationToken"]"#)
            .map_err(|e| anyhow!("RequestVerificationToken bulunamadı: {}", e))?;

        let token = document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("value"))
            .ok_or_else(|| anyhow::anyhow!("CSRF token bulunamadı"))?
            .to_string();

        Ok(token)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<(Option<String>, String)> {
        self.client.get("https://aksis.iuc.edu.tr/").send()?;

        let login_url = "https://aksis.iuc.edu.tr/Account/LogOn?returnUrl=%2F";
        let url = Url::parse(login_url)?;
        let res = self.client.get(login_url).send()?;
        let html = res.text()?;
        let csrf_token = self.get_csrf(&html)?;

        let payload = [
            ("UserName", username),
            ("Password", password),
            ("__RequestVerificationToken", &csrf_token),
        ];

        if let Some(cached) = &self.cache.aksis_cookie {
            self.jar
                .add_cookie_str(&format!("AKSISAutKH={}", cached), &url);
        } else {
            self.jar
                .add_cookie_str("AspxAutoDetectCookieSupport=1", &url);
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            header::REFERER,
            HeaderValue::from_static("https://aksis.iuc.edu.tr/Account/LogOn?ReturnUrl=%2f"),
        );

        let response = self
            .client
            .post(login_url)
            .headers(headers)
            .form(&payload)
            .send()?;

        let text = response.text()?;
        let csrf_token = self.get_csrf(&text)?;

        if text.contains("Kullanıcı Adı veya Şifre Hatalı") {
            self.cache.clear();
            bail!("Login failed. Try again")
        }

        self.cache.username = Some(username.to_string());
        self.cache.password = Some(password.to_string());
        self.cache.save();

        let cookies_val = self.jar.cookies(&url);
        let auth_cookie = cookies_val.and_then(|header| {
            header.to_str().ok().and_then(|s| {
                s.split("; ")
                    .find(|part| part.starts_with(".OGRISFormAuth="))
                    .map(|part| part.trim_start_matches(".OGRISFormAuth=").to_string())
            })
        });

        Ok((auth_cookie, csrf_token))
    }

    pub fn send_sms(&mut self, sms_code: &str, csrf_token: &str) -> Result<String> {
        let sms_url = "https://aksis.iuc.edu.tr/Account/LoginSmsmDogrula";
        let url = Url::parse(sms_url)?;
        let payload = [
            ("smsCode", sms_code),
            ("remember2fa", "true"),
            ("__RequestVerificationToken", csrf_token),
        ];

        let mut headers = HeaderMap::new();
        headers.insert(
            header::REFERER,
            HeaderValue::from_static("https://aksis.iuc.edu.tr/Account/LogOn?ReturnUrl=%2f"),
        );

        self.client.post(sms_url).form(&payload).send()?;

        let cookies_val = self.jar.cookies(&url);

        if let Some(header) = &cookies_val {
            if let Ok(s) = header.to_str() {
                if let Some(val) = s.split("; ").find(|part| part.starts_with("AKSISAutKH=")) {
                    let val = val.trim_start_matches("AKSISAutKH=");
                    self.cache.aksis_cookie = Some(val.to_string());
                    self.cache.save();
                }
            }
        }

        if let Some(header) = &cookies_val {
            if let Ok(s) = header.to_str() {
                if let Some(val) = s
                    .split("; ")
                    .find(|part| part.starts_with(".OGRISFormAuth="))
                {
                    return Ok(val.trim_start_matches(".OGRISFormAuth=").to_string());
                }
            }
        }

        bail!("SMS verification failed.");
    }
}
