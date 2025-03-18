use crate::style::{
    Style, StyleProperty, StylePropertyKey, TaffyStyleProperty, TaffyStylePropertyKey,
};
use crate::ViewId;
use bumpalo::collections::Vec;
use bumpalo::Bump;
use hashbrown::HashMap;
use taffy::{LengthPercentage, NodeId, Point, TaffyTree};

pub struct StyleComputeContext<'bump> {
    pub bump: &'bump Bump,
    pub stack: Vec<
        'bump,
        (
            HashMap<StylePropertyKey, StyleProperty, hashbrown::DefaultHashBuilder, &'bump Bump>,
            bool,
        ),
    >,
    pub style: HashMap<StylePropertyKey, StyleProperty, hashbrown::DefaultHashBuilder, &'bump Bump>,
    pub dirty: bool,
}

impl<'bump> StyleComputeContext<'bump> {
    pub fn new(bump: &'bump Bump) -> StyleComputeContext<'bump> {
        Self {
            bump,
            stack: Vec::new_in(bump),
            style: HashMap::<_, _, _, &'bump Bump>::new_in(bump),
            dirty: false,
        }
    }

    pub fn push(&mut self) {
        self.stack.push((self.style.clone(), self.dirty));
    }

    pub fn pop(&mut self) {
        if let Some((style, dirty)) = self.stack.pop() {
            self.style = style;
            self.dirty = dirty;
        }
    }
}

fn compute_style(id: &ViewId, ctx: &mut StyleComputeContext) {
    let state = id.state();
    let node = id.node();

    let mut layout_changed = false;
    let mut style_changed = false;
    let mut composite_changed = false;

    let mut style_props = HashMap::<StylePropertyKey, StyleProperty>::new();
    let mut taffy_style_props = HashMap::<TaffyStylePropertyKey, TaffyStyleProperty>::new();

    let styles = state.borrow().styles.clone();
    for style_builder in styles.into_iter().rev() {
        if let Some(style_builder) = style_builder {
            for (key, value) in style_builder.taffy_style_props.into_iter().rev() {
                if !taffy_style_props.contains_key(&key) {
                    taffy_style_props.insert(key, value);
                }
            }
            for (key, value) in style_builder.style_props.into_iter().rev() {
                if !style_props.contains_key(&key) {
                    style_props.insert(key, value);
                }
            }
        }
    }

    for (key, value) in &style_props {
        if key.inherited() {
            if ctx.style.contains_key(key) {
                ctx.style.remove(key);
            }
            ctx.style.insert(key.clone(), value.clone());
        }
    }

    for (key, value) in TaffyStyleProperty::initial() {
        if !taffy_style_props.contains_key(&key) {
            taffy_style_props.insert(key, value);
        }
    }

    let mut taffy_style = taffy::Style::default();
    for (_, value) in taffy_style_props {
        value.assign_to(&mut taffy_style);
    }

    if *id.taffy().borrow().style(node).unwrap() != taffy_style {
        layout_changed = true;
        id.taffy()
            .borrow_mut()
            .set_style(node, taffy_style)
            .unwrap();
    }

    let mut style = Style::default();

    for (key, value) in StyleProperty::initial() {
        if !style_props.contains_key(&key) {
            value.assign_to(&mut style);
        }
    }

    for (key, value) in ctx.style.iter() {
        if !style_props.contains_key(key) && key.inherited() {
            value.assign_to(&mut style);
        }
    }

    for (_, value) in style_props.iter() {
        value.assign_to(&mut style);
    }

    state.borrow_mut().style = style;


}

pub fn compute_style_recursive(id: &ViewId, ctx: &mut StyleComputeContext) {
    ctx.push();

    let state = id.state();
    // ctx.dirty = ctx.dirty || state.borrow().style_inherited_dirty;
    if ctx.dirty || state.borrow().style_dirty || true {
        compute_style(id, ctx);
        let mut state = state.borrow_mut();
        state.style_dirty = false;
        // state.style_cache = Some(ctx.style.clone().into_iter().collect());
    } else {
       // ctx.style = state.borrow().style_cache.clone().into_iter().collect();
    }

    for child in id.children().iter() {
        compute_style_recursive(child, ctx);
    }

    ctx.pop();
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
