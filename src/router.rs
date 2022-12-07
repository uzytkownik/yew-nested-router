use crate::scope::ScopeContext;
use crate::target::Target;
use gloo_history::{AnyHistory, BrowserHistory, History, HistoryListener, Location};
use std::fmt::Debug;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct RouterContext<T>
where
    T: Target,
{
    pub(crate) scope: Rc<ScopeContext<T>>,
    // The active target
    pub active_target: Option<T>,
}

impl<T> RouterContext<T>
where
    T: Target,
{
    pub fn go(&self, target: T) {
        self.scope.go(target);
    }

    /// Check if the provided target is the active target
    pub fn is_same(&self, target: &T) -> bool {
        match &self.active_target {
            Some(current) => current == target,
            None => false,
        }
    }

    pub fn is_active(&self, target: &T) -> bool {
        // FIXME: fix this
        self.is_same(target)
    }

    /// Get the active target, this may be [`None`], in the case this branch doesn't have an
    /// active target.
    pub fn active(&self) -> &Option<T> {
        &self.active_target
    }
}

/// Properties for the [`Router`] component.
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct RouterProps<T>
where
    T: Target,
{
    /// The content to render.
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub default: Option<T>,
}

#[derive(Debug)]
#[doc(hidden)]
pub enum Msg<T: Target> {
    RouteChanged(Location),
    ChangeTarget(T),
}

/// Top-level router component.
pub struct Router<T: Target> {
    history: AnyHistory,
    _listener: HistoryListener,
    target: Option<T>,

    scope: Rc<ScopeContext<T>>,
    router: RouterContext<T>,
}

impl<T> Component for Router<T>
where
    T: Target + 'static,
{
    type Message = Msg<T>;
    type Properties = RouterProps<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let history = AnyHistory::Browser(BrowserHistory::new());

        let cb = ctx.link().callback(Msg::RouteChanged);

        let target =
            Self::parse_location(history.location()).or_else(|| ctx.props().default.clone());

        let listener = {
            let history = history.clone();
            history.clone().listen(move || {
                cb.emit(history.location());
            })
        };

        let (scope, router) = Self::build_context(&target, ctx);

        Self {
            history,
            _listener: listener,
            target,
            scope,
            router,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        // log::debug!("update: {msg:?}");

        match msg {
            Msg::RouteChanged(location) => {
                let target = Self::parse_location(location).or_else(|| ctx.props().default.clone());
                if target != self.target {
                    self.target = target;
                    self.sync_context(ctx);
                    return true;
                }
            }
            Msg::ChangeTarget(target) => {
                // log::debug!("Pushing state: {:?}", request.path);
                let route = format!("/{}", target.render_path().join("/"));
                self.history.push(route);
            }
        }

        false
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.sync_context(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let scope = self.scope.clone();
        let router = self.router.clone();

        html! (
            <ContextProvider<ScopeContext<T>> context={(*scope).clone()}>
                <ContextProvider<RouterContext<T >> context={router}>
                    { for ctx.props().children.iter() }
                </ContextProvider<RouterContext<T >>>
            </ContextProvider<ScopeContext<T>>>
        )
    }
}

impl<T: Target> Router<T> {
    fn parse_location(location: Location) -> Option<T> {
        let path: Vec<&str> = location.path().split('/').skip(1).collect();
        // log::debug!("Path: {path:?}");
        let target = T::parse_path(&path);
        // log::debug!("New target: {target:?}");
        target
    }

    fn sync_context(&mut self, ctx: &Context<Self>) {
        let (scope, router) = Self::build_context(&self.target, ctx);
        self.scope = scope;
        self.router = router;
    }

    fn build_context(
        target: &Option<T>,
        ctx: &Context<Self>,
    ) -> (Rc<ScopeContext<T>>, RouterContext<T>) {
        let scope = Rc::new(ScopeContext {
            upwards: ctx.link().callback(Msg::ChangeTarget),
        });

        let router = RouterContext {
            scope: scope.clone(),
            active_target: target.clone(),
        };

        (scope, router)
    }
}

#[hook]
pub fn use_router<T>() -> Option<RouterContext<T>>
where
    T: Target + 'static,
{
    use_context()
}
