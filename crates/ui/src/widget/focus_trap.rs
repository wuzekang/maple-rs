use crate::event::use_key_down_event;
use crate::mutation_observer::{MutationObserver, ObserveOptions};
use crate::root::AppContext;
use crate::style::Styleable;
use crate::view_tuple::ViewTuple;
use crate::{view, Element, Interactive, View, ViewId};
use reactive::{create_effect, on_cleanup, provide_context, use_context, with_scope, Scope};
use sdl3_sys::everything::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

#[derive(Clone)]
struct FocusTrapContext {
    parent: Option<ViewId>,
    id: ViewId,
    active: bool,
    children: HashMap<ViewId, Rc<RefCell<FocusTrapContext>>>,
    sequence: Vec<ViewId>,
    index: HashMap<ViewId, usize>,
}

impl FocusTrapContext {
    pub fn new(id: ViewId) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent: None,
            id,
            active: true,
            children: Default::default(),
            sequence: Default::default(),
            index: Default::default(),
        }))
    }

    pub fn create_child(&mut self, id: ViewId) -> Rc<RefCell<Self>> {
        let context = Rc::new(RefCell::new(Self {
            parent: Some(self.id),
            id,
            active: true,
            children: Default::default(),
            sequence: Default::default(),
            index: Default::default(),
        }));
        self.children.insert(id, context.clone());
        context
    }

    pub fn remove_child(&mut self, id: ViewId) -> Option<Rc<RefCell<FocusTrapContext>>> {
        self.children.remove(&id)
    }

    pub fn rest(&mut self) {
        self.reset_sequence(self.id);
    }

    pub fn next(&self, forward: bool) {
        let ctx = use_context::<AppContext>().unwrap();
        let focused = *ctx.focused.borrow();

        let traps = self
            .sorted_active_traps()
            .into_iter()
            .map(|(_, trap)| trap)
            .collect::<Vec<_>>();

        if traps.is_empty() {
            return;
        }

        if focused.is_none() || traps.len() == 1 {
            if traps[0].sequence.is_empty() {
                return;
            }
            let target = if focused.is_none() {
                traps[0].sequence[0]
            } else {
                let focused = focused.unwrap();
                let seq = traps[0]
                    .index
                    .get(&focused)
                    .map(|seq| {
                        (seq + if forward { 1 } else { traps.len() - 1 }) % traps[0].sequence.len()
                    })
                    .unwrap_or(0);
                traps[0].sequence[seq]
            };
            ctx.focus(target);
            return;
        }

        let focused = focused.unwrap();

        for (i, item) in traps.iter().enumerate() {
            if !item.index.contains_key(&focused) {
                continue;
            }
            let seq = item.index[&focused];
            let target = if forward {
                if seq < item.sequence.len() - 1 {
                    item.sequence.get(seq + 1)
                } else {
                    let next_trap = (i + 1) % traps.len();
                    traps[next_trap].sequence.first()
                }
            } else if seq > 0 {
                item.sequence.get(seq - 1)
            } else {
                let next_trap = (i + traps.len() - 1) % traps.len();
                traps[next_trap].sequence.last()
            }
            .cloned()
            .unwrap();
            ctx.focus(target);
            return;
        }

        if traps[0].sequence.is_empty() {
            return;
        }
        let target = {
            let seq = traps[0]
                .index
                .get(&focused)
                .map(|seq| {
                    (seq + if forward { 1 } else { traps.len() - 1 }) % traps[0].sequence.len()
                })
                .unwrap_or(0);
            traps[0].sequence[seq]
        };
        ctx.focus(target);
    }

    pub fn sorted_active_traps(&self) -> Vec<(Vec<usize>, FocusTrapContext)> {
        let active_traps = self.active_traps();
        let root = self.id;
        let mut active_traps = active_traps
            .into_iter()
            .map(|item| (item.id.index_path(root), item))
            .collect::<Vec<_>>();
        active_traps.sort_by_key(|(path, _)| path.clone());
        active_traps
    }
    pub fn active_traps(&self) -> Vec<FocusTrapContext> {
        let mut traps = vec![];
        for context in self.children.values() {
            traps.append(&mut context.borrow().active_traps())
        }
        if self.children.is_empty() && self.active {
            traps.push(self.clone());
        }
        traps
    }

    pub fn reset_sequence(&mut self, root: ViewId) {
        let mut sequence: BTreeMap<(i32, Vec<usize>), ViewId> = Default::default();
        self.update_sequence(&root, &vec![], &mut sequence);
        self.sequence = sequence.values().cloned().collect();
        self.index = self
            .sequence
            .iter()
            .enumerate()
            .map(|(i, v)| (*v, i))
            .collect();
    }

    fn update_sequence(
        &mut self,
        node: &ViewId,
        path: &Vec<usize>,
        sequence: &mut BTreeMap<(i32, Vec<usize>), ViewId>,
    ) {
        let mut path = path.clone();
        path.push(node.state().borrow().index);
        if let Some(tab_index) = node.state().borrow().tab_index {
            sequence.insert((tab_index, path.clone()), *node);
        }
        for child in node.children().iter() {
            self.update_sequence(child, &path, sequence);
        }
    }
}

pub struct FocusTrap {
    root: View,
    context: Rc<RefCell<FocusTrapContext>>,
}

impl Default for FocusTrap {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusTrap {
    pub fn new() -> Self {
        let root = view();

        let id = root.id();

        let context = if let Some(context) = use_context::<Rc<RefCell<FocusTrapContext>>>() {
            on_cleanup({
                let context = context.clone();
                move || {
                    context.borrow_mut().remove_child(id);
                }
            });
            context.borrow_mut().create_child(id)
        } else {
            FocusTrapContext::new(id)
        };

        let observer = MutationObserver::new({
            let context = context.clone();
            move || {
                context.borrow_mut().rest();
            }
        });

        observer.observe(
            id,
            ObserveOptions {
                attributes: true,
                child_list: true,
                subtree: true,
            },
        );

        on_cleanup(move || {
            observer.disconnect();
        });

        Self { root, context }
    }

    pub fn children<VT: ViewTuple + 'static, VF: FnOnce() -> VT>(self, children: VF) -> Self {
        let Self { root, context } = self;

        if context.borrow().parent.is_none() {
            use_key_down_event({
                let context = context.clone();
                move |event| {
                    if event.key == SDLK_TAB {
                        let forward = event.r#mod & (SDL_KMOD_LSHIFT | SDL_KMOD_RSHIFT) == 0;
                        context.borrow_mut().next(forward)
                    }
                }
            });
        }

        let children = with_scope(Scope::current().create_child(), {
            let context = context.clone();
            || {
                provide_context(context);
                children()
            }
        });

        Self {
            root: root.children(children),
            context,
        }
    }

    pub fn active(self, active: impl Fn() -> bool + 'static) -> Self {
        let context = self.context.clone();
        create_effect(move |_| {
            context.borrow_mut().active = active();
        });
        self
    }
}

pub fn focus_trap() -> FocusTrap {
    FocusTrap::new()
}

impl Element for FocusTrap {
    fn id(&self) -> ViewId {
        self.root.id()
    }

    fn name(&self) -> String {
        format!("FocusTrap active={}", self.context.borrow().active)
    }
}

impl Interactive for FocusTrap {}
impl Styleable for FocusTrap {}
