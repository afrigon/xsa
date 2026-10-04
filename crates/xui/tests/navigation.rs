use std::cell::RefCell;
use std::rc::Rc;

use xui::{
    Action, AnyViewController, Context, Controller, DismissKey, DrawList, EmptyView, Environment, Interface,
    LayoutContext, Navigation, NavigationController, NavigationDelegate, NavigationOperation, NoTransition, Node,
    NodeLayout, Presentation, Rect, Size, SizeProposal, Transition, UpdateContext, View, ViewController,
};

const VIEWPORT: Size = Size {
    width: 400.0,
    height: 300.0,
};
const FADE_SECONDS: f32 = 0.25;

type Log = Rc<RefCell<Vec<String>>>;

// A screen that records each frame it is drawn, and how many frames its state has seen.
struct ScreenView {
    name: &'static str,
    log: Log,
}

struct ScreenLayout {
    name: &'static str,
    log: Log,
}

impl NodeLayout for ScreenLayout {
    fn size_that_fits(
        &mut self,
        _proposal: SizeProposal,
        _children: &mut [Node],
        _context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        Ok(Size::default())
    }

    fn place(
        &mut self,
        _bounds: Rect,
        _children: &mut [Node],
        _context: &mut LayoutContext,
        _draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        self.log.borrow_mut().push(format!("{} drawn", self.name));
        Ok(())
    }
}

impl View for ScreenView {
    fn body(&self, _context: &Context) -> impl View {
        EmptyView
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let frames = node.context(&context.environment).state(|| 0_u32);
        frames.update(|frames| *frames += 1);
        self.log
            .borrow_mut()
            .push(format!("{} frame {}", self.name, frames.get()));
        node.layout_mut(|| ScreenLayout {
            name: self.name,
            log: self.log.clone(),
        });
        node.update_children(&[], context)
    }
}

struct Screen {
    name: &'static str,
    presentation: Presentation,
    log: Log,
}

impl Screen {
    fn new(name: &'static str, presentation: Presentation, log: &Log) -> Screen {
        Screen {
            name,
            presentation,
            log: log.clone(),
        }
    }

    fn record(&self, event: &str) {
        self.log.borrow_mut().push(format!("{} {event}", self.name));
    }
}

impl ViewController for Screen {
    fn root(&self, _this: &Controller<Self>) -> impl View {
        ScreenView {
            name: self.name,
            log: self.log.clone(),
        }
    }

    fn presentation(&self) -> Presentation {
        self.presentation
    }

    fn did_load(&mut self) {
        self.record("loaded");
    }

    fn did_unload(&mut self) {
        self.record("unloaded");
    }

    fn did_appear(&mut self) {
        self.record("appeared");
    }

    fn did_disappear(&mut self) {
        self.record("disappeared");
    }
}

fn navigation_with(root: Screen) -> NavigationController {
    let mut navigation = NavigationController::new();
    navigation.set_root(root);
    navigation
}

fn render(interface: &mut Interface, navigation: &NavigationController, log: &Log) -> Vec<String> {
    log.borrow_mut().clear();
    interface
        .render(&navigation.view(), VIEWPORT, &Environment::default())
        .unwrap();
    take(log)
}

fn take(log: &Log) -> Vec<String> {
    std::mem::take(&mut *log.borrow_mut())
}

fn drawn(events: &[String]) -> Vec<&str> {
    events.iter().filter_map(|event| event.strip_suffix(" drawn")).collect()
}

#[test]
fn a_full_screen_push_hides_the_screen_below_after_its_transition() {
    let log = Log::default();
    let mut navigation = navigation_with(Screen::new("menu", Presentation::FullScreen, &log));
    let mut interface = Interface::new();
    navigation.push(Screen::new("game", Presentation::FullScreen, &log));
    assert_eq!(take(&log), ["menu loaded", "menu appeared", "game loaded"]);

    assert_eq!(drawn(&render(&mut interface, &navigation, &log)), ["menu", "game"]);
    navigation.advance(FADE_SECONDS);
    assert!(!navigation.is_transitioning());
    assert_eq!(take(&log), ["menu disappeared", "game appeared"]);
    assert_eq!(drawn(&render(&mut interface, &navigation, &log)), ["game"]);
}

#[test]
fn an_overlay_leaves_the_screen_below_drawn() {
    let log = Log::default();
    let mut navigation = navigation_with(Screen::new("game", Presentation::FullScreen, &log));
    let mut interface = Interface::new();
    navigation.push(Screen::new("pause", Presentation::Overlay, &log));
    navigation.advance(FADE_SECONDS);

    assert_eq!(
        take(&log),
        ["game loaded", "game appeared", "pause loaded", "pause appeared"]
    );
    assert_eq!(drawn(&render(&mut interface, &navigation, &log)), ["game", "pause"]);
}

#[test]
fn popping_returns_to_the_screen_below_with_its_state() {
    let log = Log::default();
    let mut navigation = navigation_with(Screen::new("menu", Presentation::FullScreen, &log));
    let mut interface = Interface::new();
    render(&mut interface, &navigation, &log);
    navigation.push(Screen::new("config", Presentation::FullScreen, &log));
    navigation.advance(FADE_SECONDS);
    render(&mut interface, &navigation, &log);
    take(&log);

    assert!(navigation.pop());
    navigation.advance(FADE_SECONDS);
    assert_eq!(take(&log), ["config disappeared", "config unloaded", "menu appeared"]);
    let events = render(&mut interface, &navigation, &log);
    assert!(events.contains(&"menu frame 3".to_string()), "{events:?}");
    assert!(!navigation.pop());
}

#[test]
fn setting_the_root_unloads_the_whole_stack() {
    let log = Log::default();
    let mut navigation = navigation_with(Screen::new("game", Presentation::FullScreen, &log));
    navigation.push(Screen::new("pause", Presentation::Overlay, &log));
    navigation.advance(FADE_SECONDS);
    take(&log);

    navigation.set_root(Screen::new("menu", Presentation::FullScreen, &log));
    navigation.advance(FADE_SECONDS);

    let events = take(&log);
    for expected in ["game unloaded", "pause unloaded", "menu appeared"] {
        assert!(events.contains(&expected.to_string()), "{events:?}");
    }
    assert!(navigation.top().is::<Screen>());
}

struct InstantDelegate;

impl NavigationDelegate for InstantDelegate {
    fn transition(
        &self,
        _operation: NavigationOperation,
        _from: &dyn AnyViewController,
        _to: &dyn AnyViewController,
    ) -> Box<dyn Transition> {
        Box::new(NoTransition)
    }
}

#[test]
fn a_delegate_chooses_the_transition() {
    let log = Log::default();
    let mut navigation = navigation_with(Screen::new("menu", Presentation::FullScreen, &log));
    navigation.set_delegate(InstantDelegate);
    navigation.push(Screen::new("game", Presentation::FullScreen, &log));

    assert!(!navigation.is_transitioning());
    assert!(take(&log).ends_with(&["menu disappeared".to_string(), "game appeared".to_string()]));
}

// A list whose rows open a detail for their id, and a detail that dismisses itself.
struct ListViewController {
    navigation: Navigation,
    log: Log,
}

struct ListView {
    show_item: Action<u32>,
}

// Opens item 7 on its first frame, standing in for a click on that row.
impl View for ListView {
    fn body(&self, context: &Context) -> impl View {
        let opened = context.state(|| false);

        if !opened.get() {
            opened.set(true);
            self.show_item.perform_with(7);
        }

        EmptyView
    }
}

impl ViewController for ListViewController {
    fn root(&self, this: &Controller<Self>) -> impl View {
        ListView {
            show_item: this.action_with(Self::show_item),
        }
    }
}

impl ListViewController {
    fn show_item(&mut self, id: u32) {
        self.navigation.push(DetailViewController {
            id,
            log: self.log.clone(),
        });
    }
}

struct DetailViewController {
    id: u32,
    log: Log,
}

struct DetailView;

impl View for DetailView {
    fn body(&self, context: &Context) -> impl View {
        context.environment().get::<DismissKey>().perform();
        EmptyView
    }
}

impl ViewController for DetailViewController {
    fn root(&self, _this: &Controller<Self>) -> impl View {
        DetailView
    }

    fn did_load(&mut self) {
        self.log.borrow_mut().push(format!("detail {} loaded", self.id));
    }

    fn did_unload(&mut self) {
        self.log.borrow_mut().push(format!("detail {} unloaded", self.id));
    }
}

#[test]
fn actions_reach_their_controller_with_their_value_and_views_dismiss_themselves() {
    let log = Log::default();
    let mut navigation = NavigationController::new();
    navigation.set_delegate(InstantDelegate);
    navigation.set_root(ListViewController {
        navigation: navigation.navigation(),
        log: log.clone(),
    });
    let mut interface = Interface::new();

    interface
        .render(&navigation.view(), VIEWPORT, &Environment::default())
        .unwrap();
    navigation.advance(0.0);
    assert_eq!(take(&log), ["detail 7 loaded"]);
    assert!(navigation.top().is::<DetailViewController>());

    interface
        .render(&navigation.view(), VIEWPORT, &Environment::default())
        .unwrap();
    navigation.advance(0.0);
    assert_eq!(take(&log), ["detail 7 unloaded"]);
    assert!(navigation.top().is::<ListViewController>());
}
