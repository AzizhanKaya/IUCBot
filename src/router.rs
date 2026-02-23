use crate::{
    client::{aksis::AksisClient, obs::ObsClient},
    pages::{self, Page, PageAction, STORE},
};
use ratatui::backend::Backend;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Login,
    Dashboard,
}

pub struct Router<B: Backend> {
    current_route: Route,
    pub current_page: Box<dyn Page<B>>,
    history: Vec<Route>,
}

impl<B: Backend> Router<B> {
    pub async fn new(initial_route: Route) -> Self {
        Self {
            current_route: initial_route.clone(),
            current_page: Self::build_page(initial_route).await,
            history: Vec::new(),
        }
    }

    pub fn current(&self) -> &Route {
        &self.current_route
    }

    pub async fn push(&mut self, route: Route) {
        if route != self.current_route {
            self.history.push(self.current_route.clone());
            self.current_route = route.clone();
            self.current_page = Self::build_page(route).await;
        }
    }

    pub async fn back(&mut self) -> bool {
        if let Some(previous) = self.history.pop() {
            self.current_route = previous.clone();
            self.current_page = Self::build_page(previous).await;
            true
        } else {
            false
        }
    }

    pub async fn handle_action(&mut self, action: PageAction) {
        match action {
            PageAction::Navigate(route) => self.push(route).await,
            PageAction::Back => {
                self.back().await;
            }
            _ => {}
        }
    }

    pub async fn build_page(route: Route) -> Box<dyn Page<B>> {
        match route {
            Route::Login => {
                let client = AksisClient::new();
                Box::new(pages::login::Login::new(client)) as Box<dyn Page<B>>
            }
            Route::Dashboard => {
                let client = ObsClient::new(STORE.get("auth_cookie").unwrap().clone()).await;
                Box::new(pages::dashboard::Dashboard::new(client)) as Box<dyn Page<B>>
            }
        }
    }
}
