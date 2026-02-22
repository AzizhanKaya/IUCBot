use crossterm::event::{Event, MouseButton};
use crossterm::event::{KeyCode, KeyModifiers, MouseEventKind};
use ratatui::{
    Frame,
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::client::aksis::AksisClient;
use crate::pages::{Contains, Page, STORE};
use crate::router::{PageAction, Route};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stage {
    Creds,
    Sms,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Focus {
    Username,
    Password,
    Login,
    SmsCode,
    Verify,
}

impl Focus {
    fn next(&mut self, stage: &Stage) {
        *self = match (stage, *self) {
            (Stage::Creds, Focus::Username) => Focus::Password,
            (Stage::Creds, Focus::Password) => Focus::Login,
            (Stage::Creds, Focus::Login) => Focus::Username,
            (Stage::Sms, Focus::SmsCode) => Focus::Verify,
            (Stage::Sms, Focus::Verify) => Focus::SmsCode,
            _ => unreachable!(),
        }
    }

    fn back(&mut self, stage: &Stage) {
        *self = match (stage, *self) {
            (Stage::Creds, Focus::Password) => Focus::Username,
            (Stage::Creds, Focus::Login) => Focus::Password,
            (Stage::Creds, Focus::Username) => Focus::Login,
            (Stage::Sms, Focus::Verify) => Focus::SmsCode,
            (Stage::Sms, Focus::SmsCode) => Focus::Verify,
            _ => unreachable!(),
        }
    }
}

pub struct Login {
    client: AksisClient,
    stage: Stage,
    focus: Focus,
    username: String,
    password: String,
    sms_code: String,
    csrf_token: String,
    layout: Layout,
    error: Option<String>,
    is_loading: bool,
    needs_login: bool,
    needs_verify: bool,
    action_ready: bool,
}

impl Login {
    pub fn new(client: AksisClient) -> Self {
        Self {
            username: client.cache.username.clone().unwrap_or_default(),
            password: client.cache.password.clone().unwrap_or_default(),
            client,
            stage: Stage::Creds,
            focus: Focus::Username,
            sms_code: String::new(),
            csrf_token: String::new(),
            layout: Layout::new(Rect::default(), &Stage::Creds),
            error: None,
            is_loading: false,
            needs_login: false,
            needs_verify: false,
            action_ready: false,
        }
    }
}

pub struct Layout {
    size: Rect,
    box_area: Rect,
    username_area: Rect,
    password_area: Rect,
    button_area: Rect,
    sms_area: Rect,
    verify_area: Rect,
}

impl Layout {
    pub fn new(size: Rect, stage: &Stage) -> Self {
        let box_width = 50;
        let box_height = 11;
        let area = Rect {
            x: (size.width.saturating_sub(box_width)) / 2,
            y: (size.height.saturating_sub(box_height)) / 2,
            width: box_width,
            height: box_height,
        };

        let inner_height = area.height.saturating_sub(2);
        let input_height = 3;
        let elements = match stage {
            Stage::Creds => 3,
            Stage::Sms => 2,
        };

        let padding = (inner_height.saturating_sub(input_height * elements)) / 2;

        let box_area = area;

        let username_area = Rect {
            x: area.x + 2,
            y: area.y + 1 + padding,
            width: area.width - 4,
            height: input_height,
        };
        let password_area = Rect {
            x: username_area.x,
            y: username_area.y + input_height,
            width: username_area.width,
            height: input_height,
        };
        let button_area = Rect {
            x: username_area.x,
            y: password_area.y + input_height,
            width: username_area.width,
            height: input_height,
        };
        let sms_area = Rect {
            x: area.x + 2,
            y: area.y + 1 + padding,
            width: area.width - 4,
            height: input_height,
        };

        let verify_area = Rect {
            x: area.x + 2,
            y: sms_area.y + input_height + 1,
            width: area.width - 4,
            height: input_height,
        };

        Self {
            size,
            box_area,
            username_area,
            password_area,
            button_area,
            sms_area,
            verify_area,
        }
    }
}

impl<B: Backend> Page<B> for Login {
    fn render(&mut self, frame: &mut Frame<B>) {
        if self.needs_login || self.needs_verify {
            self.action_ready = true;
        }

        let size = frame.size();

        if self.layout.size != size {
            self.layout = Layout::new(size, &self.stage);
        }

        match self.stage {
            Stage::Creds => self.render_creds(frame),
            Stage::Sms => self.render_sms(frame),
        }
    }

    fn handle_event(&mut self, event: Event) -> PageAction {
        if self.is_loading {
            return PageAction::None;
        }

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Down | KeyCode::Tab | KeyCode::Right => {
                    self.focus.next(&self.stage);
                }
                KeyCode::Up | KeyCode::Left => {
                    self.focus.back(&self.stage);
                }

                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        if c == 'c' || c == 'd' {
                            return PageAction::Exit;
                        }
                    } else {
                        match self.focus {
                            Focus::Username => self.username.push(c),
                            Focus::Password => self.password.push(c),
                            Focus::Login => {}
                            Focus::SmsCode => self.sms_code.push(c),
                            Focus::Verify => {}
                        }
                    }
                }
                KeyCode::Backspace => match self.focus {
                    Focus::Username => {
                        self.username.pop();
                    }
                    Focus::Password => {
                        self.password.pop();
                    }
                    Focus::Login => {}
                    Focus::SmsCode => {
                        self.sms_code.pop();
                    }
                    Focus::Verify => {}
                },

                KeyCode::Enter => {
                    if self.focus == Focus::Login
                        && !self.username.is_empty()
                        && !self.password.is_empty()
                    {
                        self.is_loading = true;
                        self.needs_login = true;
                        self.action_ready = false;
                        self.error = None;
                        return PageAction::None;
                    }

                    if self.focus == Focus::Verify && !self.sms_code.is_empty() {
                        self.is_loading = true;
                        self.needs_verify = true;
                        self.action_ready = false;
                        self.error = None;
                        return PageAction::None;
                    }
                }

                KeyCode::Esc => {
                    return PageAction::Exit;
                }
                _ => {}
            },
            Event::Mouse(e) => {
                if let MouseEventKind::Down(MouseButton::Left) = e.kind {
                    match self.stage {
                        Stage::Creds => {
                            if self.layout.username_area.contains(e.row, e.column) {
                                self.focus = Focus::Username;
                            } else if self.layout.password_area.contains(e.row, e.column) {
                                self.focus = Focus::Password;
                            } else if self.layout.button_area.contains(e.row, e.column) {
                                self.focus = Focus::Login;
                            }
                        }
                        Stage::Sms => {
                            if self.layout.sms_area.contains(e.row, e.column) {
                                self.focus = Focus::SmsCode;
                            } else if self.layout.verify_area.contains(e.row, e.column) {
                                self.focus = Focus::Verify;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        PageAction::None
    }

    fn tick(&mut self) -> PageAction {
        if self.action_ready {
            if self.needs_login {
                self.needs_login = false;
                self.action_ready = false;

                let auth_cookie = match self.client.login(&self.username, &self.password) {
                    Ok((Some(cookie), _)) => cookie,
                    Ok((None, csrf_token)) => {
                        self.csrf_token = csrf_token;
                        self.stage = Stage::Sms;
                        self.focus = Focus::SmsCode;
                        self.is_loading = false;
                        return PageAction::None;
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                        self.is_loading = false;
                        return PageAction::None;
                    }
                };

                STORE.insert("auth_cookie".to_string(), auth_cookie);
                self.is_loading = false;
                return PageAction::Navigate(Route::Dashboard);
            } else if self.needs_verify {
                self.needs_verify = false;
                self.action_ready = false;

                let Ok(auth_cookie) = self.client.send_sms(&self.sms_code, &self.csrf_token) else {
                    self.error = Some("SMS code error".to_string());
                    self.is_loading = false;
                    return PageAction::None;
                };

                STORE.insert("auth_cookie".to_string(), auth_cookie);
                self.is_loading = false;
                return PageAction::Navigate(Route::Dashboard);
            }
        }
        PageAction::None
    }
}

impl Login {
    fn render_creds<B: Backend>(&self, frame: &mut Frame<B>) {
        let layout = &self.layout;

        let login_block = Block::default()
            .title("Giriş")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        frame.render_widget(login_block, layout.box_area);

        let username_block = Block::default()
            .borders(Borders::ALL)
            .title("Kullanıcı Adı")
            .style(if self.focus == Focus::Username {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
        let username_paragraph = Paragraph::new(self.username.clone())
            .block(username_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(username_paragraph, layout.username_area);

        let password_block = Block::default().borders(Borders::ALL).title("Şifre").style(
            if self.focus == Focus::Password {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        );
        let password_paragraph = Paragraph::new("*".repeat(self.password.len()))
            .block(password_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(password_paragraph, layout.password_area);

        let button_style = if self.focus == Focus::Login {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::Green)
        };

        let button_text = if self.is_loading {
            "   Giriş Yapılıyor...   "
        } else {
            "   Giriş Yap   "
        };

        let login_button = Paragraph::new(button_text)
            .style(button_style)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(login_button, layout.button_area);

        if let Some(error) = &self.error {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);

            let area = Rect {
                x: layout.box_area.x,
                y: layout.box_area.y + layout.box_area.height + 1,
                width: layout.box_area.width,
                height: 1,
            };
            frame.render_widget(error_paragraph, area);
        }
    }

    fn render_sms<B: Backend>(&self, frame: &mut Frame<B>) {
        let layout = &self.layout;

        let sms_block = Block::default()
            .title("SMS Doğrulama")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        frame.render_widget(sms_block, layout.box_area);

        let sms_code_block = Block::default()
            .borders(Borders::ALL)
            .title("SMS Kodu")
            .style(if self.focus == Focus::SmsCode {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let sms_code_paragraph = Paragraph::new(self.sms_code.clone())
            .block(sms_code_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(sms_code_paragraph, layout.sms_area);

        let button_style = if self.focus == Focus::Verify {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::Green)
        };

        let button_text = if self.is_loading {
            "   Doğrulanıyor...   "
        } else {
            "   Doğrula   "
        };

        let verify_button = Paragraph::new(button_text)
            .style(button_style)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        frame.render_widget(verify_button, layout.verify_area);

        if let Some(error) = &self.error {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);

            let area = Rect {
                x: layout.box_area.x,
                y: layout.box_area.y + layout.box_area.height + 1,
                width: layout.box_area.width,
                height: 1,
            };
            frame.render_widget(error_paragraph, area);
        }
    }
}
