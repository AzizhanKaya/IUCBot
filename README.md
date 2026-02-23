# IUCBot: TUI ve Otomasyon İstemcisi

IUCBot, İstanbul Üniversitesi - Cerrahpaşa öğrencileri için geliştirilmiş, terminal tabanlı (TUI) bir AKSİS ve OBS (Öğrenci Bilgi Sistemi) istemcisi ve otomasyon aracıdır. `rust`, `crossterm` ve `ratatui` kullanılarak geliştirilmiştir.

Uygulama üzerinden üniversite sistemine tarayıcıya ihtiyaç duymadan terminalden hızlıca giriş yapabilir ve otomasyon özelliklerinden yararlanabilirsiniz.


## TODO / Geliştirme Durumu

- [x] Özlük Bilgileri
- [x] Ders Notları
  - [ ] Harf Notları
- [x] Ders Seçimi
- [ ] Anket çözme
- [ ] Notları bildirim gönderme
- [ ] Ders dosya senkronizasyonu

## Demo Video

https://github.com/user-attachments/assets/58fa60fb-a868-4582-818f-3eb73b520b3f

## Kurulum ve Çalıştırma

Projeyi derleyip çalıştırabilmek için [Rust](https://rustup.rs/) yüklü olmalıdır.

```bash
git clone https://github.com/AzizhanKaya/IUCBot
cd IUCBot

cargo run --release
```

## Kontroller (Klavye Kısayolları)

Uygulama tamamen klavye ile kontrol edilecek şekilde tasarlanmıştır:

- `Tab` : Dashboard içindeki sekmeler arasında gezinmeyi sağlar.
- `Ok Tuşları (↑/↓)` & `Tab/Shift+Tab` : Giriş formlarında, listelerde geçişi sağlar.
- `Enter` : Seçimleri onaylama veya listeleri/dersleri genişletme (açılır kapanır listeler).
- `CTRL+C`, `q` veya `Esc` : Uygulamayı güvenli bir şekilde kapatır ve terminale döner.

## Mimari ve Teknolojiler

- **[Rust](https://www.rust-lang.org/):** Yüksek performans ve güvenilirlik.
- **[Ratatui](https://ratatui.rs/) & [Crossterm](https://github.com/crossterm-rs/crossterm):** Temel TUI bileşenleri ve terminal etkileşimi.
- **[Tokio](https://tokio.rs/):** Sayfa geçişlerinde direkt polling yapabilen lock-free async ekosistem.
- **[Reqwest](https://docs.rs/reqwest/latest/reqwest/):** TLS, oturum ve HTTP ağ çağrı yönetimi.
- **[Scraper](https://docs.rs/scraper/latest/scraper/):** HTML sayfalarını analiz etmek ve terminale uygun modele dönüştürmek için.

---

_Uyarı: Bu uygulama resmi bir kurum (İÜC) uygulaması değildir. Açık kaynaklı bireysel bir geliştirici / otomasyon aracıdır._
