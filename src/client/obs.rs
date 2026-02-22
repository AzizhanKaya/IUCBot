use anyhow::{Context, Result};
use reqwest::{Url, blocking::Client, cookie::Jar};
use serde::Deserialize;
use std::{
    io::{self, Write},
    sync::Arc,
};

#[derive(Deserialize, Debug)]
pub struct Exam {
    #[serde(rename = "SinavTuru")]
    pub name: String,
    #[serde(rename = "Notu")]
    pub grade: String,
    #[serde(rename = "EtkiOrani")]
    pub weight: String,
    #[serde(rename = "SinavYayinlanmaTarihiString")]
    pub date: String,
}

#[derive(Deserialize, Debug)]
pub struct Course {
    #[serde(rename = "Key")]
    pub name: String,
    #[serde(rename = "Items")]
    pub exams: Vec<Exam>,
    pub grade_scale: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ExamResults {
    #[serde(rename = "Data")]
    pub data: Vec<Course>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct AlinabilecekDers {
    pub DersGrupID: u64,
    pub DersPlanAnaId: u64,
    pub DersPlanId: u64,
    pub SecmeliGrubu: u64,
    pub Grup: Option<String>,
    pub IntibakAID: Option<u64>,
    pub IntibakBID: Option<u64>,
    pub EnumKontenjanTuru: u64,
    pub YerineAlinanDersID: Option<u64>,
    pub SaydirilanDersID: u64,
    pub Kodu: String,
    pub DersAdi: String,
    pub Tipi: String,
    pub Selected: String,
    #[serde(skip)]
    pub is_selected: bool,
}

#[derive(Deserialize, Debug)]
pub struct AlinabilecekDerslerResponse {
    #[serde(rename = "Data")]
    pub data: Vec<AlinabilecekDers>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct KaydetResponse {
    pub MessageType: String,
    pub MessageText: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct IletisimBilgisi {
    pub IletisimTipi: String,
    pub Telefon_Mail: String,
    pub DogrulandimiStr: Option<String>,
    pub TercihEdilenMi: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct IletisimBilgileriResponse {
    pub Data: Vec<IletisimBilgisi>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct AdresBilgisi {
    pub AdresTipi: String,
    pub Adres: String,
    pub ilce: Option<String>,
    pub il: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct AdresBilgileriResponse {
    pub Data: Vec<AdresBilgisi>,
}

#[derive(Debug, Clone)]
pub struct OzlukBilgileriData {
    pub ad: String,
    pub soyad: String,
    pub tc_kimlik: String,
    pub resim_base64: String,
    pub iletisim: Vec<IletisimBilgisi>,
    pub adresler: Vec<AdresBilgisi>,
}

pub struct ObsClient {
    client: Client,
}

impl ObsClient {
    pub fn new(auth_cookie: String) -> Self {
        let jar = Arc::new(Jar::default());
        let url = Url::parse("https://obs.iuc.edu.tr/").unwrap();

        jar.add_cookie_str(&format!(".OGRISFormAuth={}", auth_cookie), &url);

        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(jar)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        client.get("https://obs.iuc.edu.tr/").send().unwrap();

        ObsClient { client }
    }

    pub fn get_exam_results(&self, year: &str, term: &str) -> Result<Vec<Course>> {
        let url =
            "https://obs.iuc.edu.tr/OgrenimBilgileri/SinavSonuclariVeNotlar/GetOgrenciSinavSonuc";
        let payload = [("group", "DersAdi-asc"), ("yil", year), ("donem", term)];

        let res = self.client.post(url).form(&payload).send()?;
        let result_text = res.text()?;

        let results: ExamResults = serde_json::from_str(&result_text)?;
        Ok(results.data)
    }

    pub fn get_alinabilecek_dersler(&self) -> Result<Vec<AlinabilecekDers>> {
        let url = "https://obs.iuc.edu.tr/DersAlma/DersAlma/GetAlinabilecekDersler";
        let payload = [("sort", ""), ("group", ""), ("filter", "")];

        let res = self.client.post(url).form(&payload).send()?;

        let result_text = res.text()?;

        if result_text.contains("HataTakvim") {
            anyhow::bail!("Ders Alma Takvimi Dışındasınız");
        }

        let mut results: AlinabilecekDerslerResponse = serde_json::from_str(&result_text)?;
        for c in &mut results.data {
            c.is_selected = c.Selected.contains("checked");
        }
        Ok(results.data)
    }

    pub fn kaydet_dersler(&self, dersler: &[AlinabilecekDers]) -> Result<String> {
        let url = "https://obs.iuc.edu.tr/DersAlma/DersAlma/Kaydet";

        let mut form_data = Vec::new();

        for (i, ders) in dersler.iter().enumerate() {
            form_data.push((
                format!("parametre[{}][DersGrupId]", i),
                ders.DersGrupID.to_string(),
            ));
            form_data.push((
                format!("parametre[{}][DersPlanAnaID]", i),
                ders.DersPlanAnaId.to_string(),
            ));
            form_data.push((
                format!("parametre[{}][DersPlanID]", i),
                ders.DersPlanId.to_string(),
            ));
            form_data.push((
                format!("parametre[{}][DersEtiket]", i),
                ders.SecmeliGrubu.to_string(),
            ));

            let etiket_adi = match ders.Grup.as_deref() {
                Some("Zorunlu") | None => "NaN".to_string(),
                Some(s) => s.to_string(),
            };
            form_data.push((format!("parametre[{}][DersEtiketAdi]", i), etiket_adi));

            form_data.push((
                format!("parametre[{}][IntibaklananAID]", i),
                ders.IntibakAID
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "NaN".to_string()),
            ));
            form_data.push((
                format!("parametre[{}][IntibaklananBID]", i),
                ders.IntibakBID
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "NaN".to_string()),
            ));
            form_data.push((
                format!("parametre[{}][EnumKontenjanTuru]", i),
                ders.EnumKontenjanTuru.to_string(),
            ));
            form_data.push((
                format!("parametre[{}][YerineAlinanDersID]", i),
                ders.YerineAlinanDersID
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "NaN".to_string()),
            ));
            form_data.push((
                format!("parametre[{}][SaydirilanDers]", i),
                ders.SaydirilanDersID.to_string(),
            ));
        }

        let res = self.client.post(url).form(&form_data).send()?;
        let result_text = res.text()?;

        let response: KaydetResponse = serde_json::from_str(&result_text)?;
        if response.MessageType == "success" {
            Ok(response.MessageText)
        } else {
            anyhow::bail!(response.MessageText)
        }
    }

    pub fn get_ozluk_bilgileri(&self) -> Result<OzlukBilgileriData> {
        let url = "https://obs.iuc.edu.tr/Profil/Ozluk";
        let res = self.client.get(url).send()?;
        let html = res.text()?;

        let document = scraper::Html::parse_document(&html);
        let fg_sel = scraper::Selector::parse(".form-group").unwrap();
        let label_sel = scraper::Selector::parse("label").unwrap();
        let span_sel = scraper::Selector::parse("span.info-input").unwrap();

        let mut ad = String::new();
        let mut soyad = String::new();
        let mut tc_kimlik = String::new();

        for fg in document.select(&fg_sel) {
            if let Some(label) = fg.select(&label_sel).next() {
                let label_text = label.text().collect::<String>().trim().to_string();
                if let Some(span) = fg.select(&span_sel).next() {
                    let span_text = span.text().collect::<String>().trim().to_string();
                    if label_text == "Adı:" {
                        ad = span_text;
                    } else if label_text == "Soyadı:" {
                        soyad = span_text;
                    } else if label_text == "Kimlik Numarası:" {
                        tc_kimlik = span_text;
                    }
                }
            }
        }

        let img_sel = scraper::Selector::parse("img[src^='data:image']").unwrap();
        let mut resim_base64 = "".to_string();
        if let Some(img) = document.select(&img_sel).next() {
            if let Some(src) = img.value().attr("src") {
                resim_base64 = src.to_string();
            }
        }

        let kisi_id = html
            .split("kisiId: '")
            .nth(1)
            .and_then(|s| s.split('\'').next())
            .context("kisiId bulunamadı")?;

        let mut iletisim = Vec::new();
        let mut adresler = Vec::new();

        if !kisi_id.is_empty() {
            let iletisim_url = "https://obs.iuc.edu.tr/Profil/Ozluk/GetIletisimBilgileri";
            let payload = [
                ("sort", ""),
                ("group", ""),
                ("filter", ""),
                ("kisiId", kisi_id),
            ];

            if let Ok(res) = self.client.post(iletisim_url).form(&payload).send() {
                if let Ok(text) = res.text() {
                    if let Ok(data) = serde_json::from_str::<IletisimBilgileriResponse>(&text) {
                        iletisim = data.Data;
                    }
                }
            }

            let adres_url = "https://obs.iuc.edu.tr/Profil/Ozluk/GetAdresBilgileri";
            if let Ok(res) = self.client.post(adres_url).form(&payload).send() {
                if let Ok(text) = res.text() {
                    if let Ok(data) = serde_json::from_str::<AdresBilgileriResponse>(&text) {
                        adresler = data.Data;
                    }
                }
            }
        }

        Ok(OzlukBilgileriData {
            ad,
            soyad,
            tc_kimlik,
            resim_base64,
            iletisim,
            adresler,
        })
    }
}
