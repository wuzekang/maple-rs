use crate::style::{
    Style, StyleProperty, StylePropertyKey, StyleTrigger, TaffyStyleProperty, TaffyStylePropertyKey,
};
use crate::ViewId;
use bumpalo::Bump;
use hashbrown::{DefaultHashBuilder, HashMap, HashSet};
use strum::IntoEnumIterator;
use taffy::{LengthPercentage, NodeId, Point, TaffyTree};

pub struct StyleComputeContext<'bump> {
    pub bump: &'bump Bump,
    pub style: HashMap<StylePropertyKey, StyleProperty, DefaultHashBuilder, &'bump Bump>,
    pub dirty: bool,
    pub style_initial: Vec<(StylePropertyKey, StyleProperty)>,
    pub taffy_style_initial: Vec<(TaffyStylePropertyKey, TaffyStyleProperty)>,
    pub visited: HashSet<ViewId, DefaultHashBuilder, &'bump Bump>,
}

impl<'bump> StyleComputeContext<'bump> {
    pub fn new(bump: &'bump Bump) -> StyleComputeContext<'bump> {
        Self {
            bump,
            style: HashMap::<_, _, _, &'bump Bump>::new_in(bump),
            dirty: false,
            visited: HashSet::new_in(bump),
            style_initial: StyleProperty::initial(),
            taffy_style_initial: TaffyStyleProperty::initial(),
        }
    }
}

fn compute_style(id: &ViewId, ctx: &mut StyleComputeContext) {
    let state = id.state();
    let node = id.node();

    let mut style_trigger = StyleTrigger::None;

    let mut style_props = HashMap::<StylePropertyKey, StyleProperty, _, _>::new_in(ctx.bump);
    let mut taffy_style_props =
        HashMap::<TaffyStylePropertyKey, TaffyStyleProperty, _, _>::new_in(ctx.bump);

    for style_builder in state.borrow().styles.iter().rev() {
        if let Some(style_builder) = style_builder.as_ref() {
            for (key, value) in style_builder.taffy_style_props.iter().rev() {
                if !taffy_style_props.contains_key(key) {
                    taffy_style_props.insert(*key, *value);
                }
            }
            for (key, value) in style_builder.style_props.iter().rev() {
                if !style_props.contains_key(key) {
                    style_props.insert(*key, value.clone());
                }
            }
        }
    }

    for (key, value) in &ctx.taffy_style_initial {
        if !taffy_style_props.contains_key(key) {
            taffy_style_props.insert(*key, *value);
        }
    }

    let mut taffy_style = taffy::Style::default();
    for (_, value) in taffy_style_props {
        value.assign_to(&mut taffy_style);
    }

    if *id.taffy().borrow().style(node).unwrap() != taffy_style {
        style_trigger = StyleTrigger::Layout;
        id.taffy()
            .borrow_mut()
            .set_style(node, taffy_style)
            .unwrap();
    }

    let mut style = Style::default();
    for (key, value) in &ctx.style_initial {
        if !style_props.contains_key(key) {
            value.assign_to(&mut style);
        }
    }

    let parent_style = id
        .parent()
        .map(|id| id.state().borrow().style.clone())
        .unwrap_or_else(Default::default);

    for key in StylePropertyKey::iter() {
        if !style_props.contains_key(&key) && key.inherited() {
            key.value(&parent_style).assign_to(&mut style);
        }
    }

    for (_, value) in style_props.iter() {
        value.assign_to(&mut style);
    }

    for key in StylePropertyKey::iter() {
        if key.value(&state.borrow().style) != key.value(&style) {
            let trigger = key.trigger();
            if trigger > style_trigger {
                style_trigger = trigger;
            }
            if !ctx.dirty && key.inherited() {
                ctx.dirty = true;
            }
        }
    }

    state.borrow_mut().style = style;

    for (key, value) in &style_props {
        if key.inherited() {
            if ctx.style.contains_key(key) {
                ctx.style.remove(key);
            }
            ctx.style.insert(key.clone(), value.clone());
        }
    }

    id.request_repaint(style_trigger);
}

pub fn compute_style_recursive(id: &ViewId, ctx: &mut StyleComputeContext) {
    if ctx.visited.contains(id) {
        return;
    }
    let dirty = ctx.dirty;
    {
        let state = id.state();
        compute_style(id, ctx);
        let mut state = state.borrow_mut();
        ctx.visited.insert(*id);

        state.style_cache.clear();
        for (key, value) in &ctx.style {
            state.style_cache.insert(key.clone(), value.clone());
        }
    }
    if ctx.dirty {
        for child in id.children().iter() {
            compute_style_recursive(child, ctx);
        }
    }
    ctx.dirty = dirty;
}

pub fn compute_layout(taffy: &mut TaffyTree, parent: NodeId, viewport: Point<f32>) {
    let children = taffy.children(parent).unwrap();
    for child in children {
        let id = ViewId(child);
        let state = id.state();
        state.borrow_mut().viewport = viewport;
        let layout = taffy.layout(child).unwrap();
        let size = layout.size;
        let location = layout.location;

        let translate = match state.borrow().style.translate {
            Point { x, y } => Point {
                x: match x {
                    LengthPercentage::Length(value) => value,
                    LengthPercentage::Percent(value) => value * size.width,
                },
                y: match y {
                    LengthPercentage::Length(value) => value,
                    LengthPercentage::Percent(value) => value * size.height,
                },
            },
        };

        let viewport = viewport + location + translate;
        compute_layout(taffy, child, viewport);
    }
}
