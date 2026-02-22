use crate::{
    client::{aksis::AksisClient, obs::ObsClient},
    pages::{self, Page, STORE},
};
use ratatui::backend::Backend;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Login,
    Dashboard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageAction {
    None,
    Exit,
    Navigate(Route),
    Back,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterResult {
    None,
    Exit,
    Error(String),
}

impl Default for PageAction {
    fn default() -> Self {
        PageAction::None
    }
}

pub struct Router<B: Backend> {
    current_route: Route,
    pub current_page: Box<dyn Page<B>>,
    history: Vec<Route>,
}

impl<B: Backend> Router<B> {
    pub fn new(initial_route: Route) -> Self {
        Self {
            current_route: initial_route.clone(),
            current_page: Self::build_page(initial_route),
            history: Vec::new(),
        }
    }

    pub fn current(&self) -> &Route {
        &self.current_route
    }

    pub fn push(&mut self, route: Route) {
        if route != self.current_route {
            self.history.push(self.current_route.clone());
            self.current_route = route.clone();
            self.current_page = Self::build_page(route);
        }
    }

    pub fn back(&mut self) -> bool {
        if let Some(previous) = self.history.pop() {
            self.current_route = previous.clone();
            self.current_page = Self::build_page(previous);
            true
        } else {
            false
        }
    }

    pub fn handle_action(&mut self, action: PageAction) -> RouterResult {
        match action {
            PageAction::None => RouterResult::None,
            PageAction::Exit => RouterResult::Exit,
            PageAction::Navigate(route) => {
                self.push(route);
                RouterResult::None
            }
            PageAction::Back => {
                self.back();
                RouterResult::None
            }
            PageAction::Error(error) => RouterResult::Error(error),
        }
    }

    pub fn build_page(route: Route) -> Box<dyn Page<B>> {
        match route {
            Route::Login => {
                let client = AksisClient::new();
                Box::new(pages::login::Login::new(client))
            }
            Route::Dashboard => {
                let client = ObsClient::new(STORE.get("auth_cookie").unwrap().clone());
                Box::new(pages::dashboard::Dashboard::new(client))
            }
        }
    }
}
