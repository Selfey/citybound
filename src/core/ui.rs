use ::monet::glium::{DisplayBuild, glutin};
use kay::{ActorSystem, ID, Individual, Recipient, Fate};
use descartes::{N, P2, P3, V3, Into2d, Shape};
use ::monet::{Renderer, Scene, GlutinFacade, MoveEye};
use ::monet::glium::glutin::{Event, MouseScrollDelta, ElementState, MouseButton};
pub use ::monet::glium::glutin::{VirtualKeyCode};
use core::geometry::AnyShape;
use ::std::collections::HashMap;

pub struct UserInterface {
    interactables_2d: HashMap<ID, (AnyShape, usize)>,
    interactables_3d: HashMap<ID, (AnyShape, usize)>,
    cursor_2d: P2,
    cursor_3d: P3,
    drag_start_2d: Option<P2>,
    drag_start_3d: Option<P3>,
    hovered_interactable: Option<ID>,
    active_interactable: Option<ID>,
    focused_interactable: Option<ID>,
    space_pressed: bool,
    shift_pressed: bool
}
impl Individual for UserInterface{}

impl UserInterface{
    fn new() -> UserInterface {
        UserInterface {
            interactables_2d: HashMap::new(),
            interactables_3d: HashMap::new(),
            cursor_2d: P2::new(0.0, 0.0),
            cursor_3d: P3::new(0.0, 0.0, 0.0),
            drag_start_2d: None,
            drag_start_3d: None,
            hovered_interactable: None,
            active_interactable: None,
            focused_interactable: None,
            space_pressed: false,
            shift_pressed: false
        }
    }
}

#[derive(Compact, Clone)]
pub enum Add{Interactable2d(ID, AnyShape, usize), Interactable3d(ID, AnyShape, usize)}

impl Recipient<Add> for UserInterface {
    fn receive(&mut self, msg: &Add) -> Fate {match *msg{
        Add::Interactable2d(_id, ref _shape, _z_index) => unimplemented!(),
        Add::Interactable3d(id, ref shape, z_index) => {
            self.interactables_3d.insert(id, (shape.clone(), z_index));
            Fate::Live
        }
    }}
}

#[derive(Copy, Clone)]
pub enum Remove{Interactable2d(ID), Interactable3d(ID)}

impl Recipient<Remove> for UserInterface {
    fn receive(&mut self, msg: &Remove) -> Fate {match *msg{
        Remove::Interactable2d(_id) => unimplemented!(),
        Remove::Interactable3d(id) => {
            self.interactables_3d.remove(&id);
            Fate::Live
        }
    }}
}

#[derive(Copy, Clone)]
pub struct Focus(pub ID);

impl Recipient<Focus> for UserInterface {
    fn receive(&mut self, msg: &Focus) -> Fate {match *msg{
        Focus(id) => {
            self.focused_interactable = Some(id);
            Fate::Live
        }
    }}
}

#[derive(Copy, Clone)]
enum Mouse{Moved(P2), Scrolled(P2), Down, Up}

use ::monet::Project2dTo3d;

#[derive(Copy, Clone)]
pub enum Event3d{
    DragStarted{at: P3},
    DragOngoing{from: P3, to: P3},
    DragFinished{from: P3, to: P3},
    DragAborted,
    HoverStarted{at: P3},
    HoverOngoing{at: P3},
    HoverStopped,
    KeyDown(VirtualKeyCode),
    KeyUp(VirtualKeyCode)
}

impl Recipient<Mouse> for UserInterface {
    fn receive(&mut self, msg: &Mouse) -> Fate {match *msg{
        Mouse::Moved(position) => {
            let delta = self.cursor_2d - position;
            if self.space_pressed {
                if self.shift_pressed {
                    Renderer::id() << MoveEye{scene_id: 0, movement: ::monet::Movement::Rotate(-delta.x/300.0)};
                    Renderer::id() << MoveEye{scene_id: 0, movement: ::monet::Movement::Zoom(-delta.y)};
                    self.cursor_2d = position;
                } else {
                    Renderer::id() << MoveEye{scene_id: 0, movement: ::monet::Movement::Shift(V3::new(-delta.y / 3.0, delta.x / 3.0, 0.0))};                
                    self.cursor_2d = position;
                }
            } else {
                self.cursor_2d = position;
                Renderer::id() << Project2dTo3d{
                    scene_id: 0,
                    position_2d: position,
                    requester: Self::id()
                };
            }
            Fate::Live
        },
        Mouse::Scrolled(delta) => {
            Renderer::id() << MoveEye{scene_id: 0, movement: ::monet::Movement::Shift(V3::new(delta.y / 5.0, -delta.x / 5.0, 0.0))};
            Fate::Live
        },
        Mouse::Down => {
            self.drag_start_2d = Some(self.cursor_2d);
            self.drag_start_3d = Some(self.cursor_3d);
            let cursor_3d = self.cursor_3d;
            self.receive(&Projected3d{position_3d: cursor_3d});
            self.active_interactable = self.hovered_interactable;
            if let Some(active_interactable) = self.active_interactable{
                active_interactable << Event3d::DragStarted{at: self.cursor_3d};
            }
            Fate::Live
        },
        Mouse::Up => {
            if let Some(active_interactable) = self.active_interactable {
                active_interactable << Event3d::DragFinished{
                    from: self.drag_start_3d.expect("active interactable but no drag start"),
                    to: self.cursor_3d
                };
            }
            self.drag_start_2d = None;
            self.drag_start_3d = None;
            self.active_interactable = None;
            Fate::Live
        }
    }}
}

#[derive(Copy, Clone)]
enum Key{Up(VirtualKeyCode), Down(VirtualKeyCode)}

impl Recipient<Key> for UserInterface {
    fn receive(&mut self, msg: &Key) -> Fate {match *msg {
        Key::Down(key_code) => {
            match key_code {
                VirtualKeyCode::Space => {
                    self.space_pressed = true;
                },
                VirtualKeyCode::LShift | VirtualKeyCode::RShift => {
                    self.shift_pressed = true;
                },
                _ => {}
            }
            self.focused_interactable.map(|interactable|
                interactable << Event3d::KeyDown(key_code)
            );
            Fate::Live
        },
        Key::Up(key_code) => {
            match key_code {
                VirtualKeyCode::Space => {
                    self.space_pressed = false;
                },
                VirtualKeyCode::LShift | VirtualKeyCode::RShift => {
                    self.shift_pressed = false;
                },
                _ => {}
            }
            self.focused_interactable.map(|interactable|
                interactable << Event3d::KeyUp(key_code)
            );
            Fate::Live
        }
    }}
}

use ::monet::Projected3d;

impl Recipient<Projected3d> for UserInterface {
    fn receive(&mut self, msg: &Projected3d) -> Fate {match *msg{
        Projected3d{position_3d} => {
            self.cursor_3d = position_3d;
            if let Some(active_interactable) = self.active_interactable {
                active_interactable << Event3d::DragOngoing{
                    from: self.drag_start_3d.expect("active interactable but no drag start"),
                    to: position_3d
                };
            } else {
                let new_hovered_interactable = self.interactables_3d.iter().filter(|&(_id, &(ref shape, _z_index))|
                    shape.contains(position_3d.into_2d())
                ).max_by_key(|&(_id, &(ref _shape, z_index))|
                    z_index
                ).map(|(id, _shape)| *id);

                if self.hovered_interactable != new_hovered_interactable {
                    if let Some(previous) = self.hovered_interactable {
                        previous << Event3d::HoverStopped;
                    }
                    if let Some(next) = new_hovered_interactable {
                        next << Event3d::HoverStarted{at: self.cursor_3d};
                    }
                } else if let Some(hovered_interactable) = self.hovered_interactable {
                    hovered_interactable << Event3d::HoverOngoing{at: self.cursor_3d};
                }
                self.hovered_interactable = new_hovered_interactable;
            }
            Fate::Live
        }
    }}
}

pub fn setup_window_and_renderer(system: &mut ActorSystem, renderables: Vec<ID>) -> GlutinFacade {
    let window = glutin::WindowBuilder::new()
        .with_title("Citybound".to_string())
        .with_dimensions(1024, 512)
        .with_multitouch()
        .with_vsync().build_glium().unwrap();

    let ui = UserInterface::new();

    system.add_individual(ui);
    system.add_inbox::<Add, UserInterface>();
    system.add_inbox::<Remove, UserInterface>();
    system.add_inbox::<Focus, UserInterface>();
    system.add_unclearable_inbox::<Mouse, UserInterface>();
    system.add_unclearable_inbox::<Key, UserInterface>();
    system.add_unclearable_inbox::<Projected3d, UserInterface>();

    let mut renderer = Renderer::new(window.clone());
    let mut scene = Scene::new();
    scene.eye.position *= 30.0;
    scene.renderables = renderables;
    renderer.scenes.insert(0, scene);

    ::monet::setup(system, renderer);

    window
}

pub fn process_events(window: &GlutinFacade) -> bool {
    let mut cleaned_events = window.poll_events().collect::<Vec<_>>();
    if let Some(last_mouse_move_event_pos) = cleaned_events.iter().rposition(|event| match *event { Event::MouseMoved(..) => true, _ => false}) {
        let mut i = 0;
        cleaned_events.retain(|event| {
            let keep = match *event {
                Event::MouseMoved(..) => i == last_mouse_move_event_pos,
                _ => true
            };
            i += 1;
            keep
        });
    }
    for event in cleaned_events {
        match event {
            Event::Closed => return false,
            Event::MouseWheel(MouseScrollDelta::PixelDelta(x, y), _) =>
                UserInterface::id() << Mouse::Scrolled(P2::new(x as N, y as N)),
            Event::MouseMoved(x, y) =>
                UserInterface::id() << Mouse::Moved(P2::new(x as N, y as N)),
            Event::MouseInput(ElementState::Pressed, MouseButton::Left) =>
                UserInterface::id() << Mouse::Down,
            Event::MouseInput(ElementState::Released, MouseButton::Left) =>
                UserInterface::id() << Mouse::Up,
            Event::KeyboardInput(ElementState::Pressed, _, Some(key_code)) =>
                UserInterface::id() << Key::Down(key_code),
            Event::KeyboardInput(ElementState::Released, _, Some(key_code)) =>
                UserInterface::id() << Key::Up(key_code),
            _ => {}
        }
    }
    true
}