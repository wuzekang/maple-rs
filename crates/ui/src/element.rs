use crate::{
    dynamic::Dynamic, fragment::Fragment, runtime::RUNTIME, sdl::Renderer, text::Text,
    view_id::ViewId,
};
use reactive::{create_memo, create_rw_signal, Memo, ReadSignal, SignalGet};
use std::{cell::RefCell, rc::Rc};
use taffy::{AvailableSpace, Size};

pub trait Element {
    fn id(&self) -> ViewId;

    fn paint(&self, cx: &Renderer) {
        for child in self.id().children() {
            child.element().paint(cx);
        }
    }
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        Size::ZERO
    }
}

pub trait IntoElement: Sized {
    type V: SignalGet<Vec<ViewId>> + 'static;
    fn into_element(self) -> Self::V;
}

impl<T: Element + 'static> IntoElement for T {
    type V = ReadSignal<Vec<ViewId>>;
    fn into_element(self) -> Self::V {
        let id = self.id();
        RUNTIME.with_borrow_mut(|r| {
            r.elements
                .insert(id.node().into(), Rc::new(self));
        });
        create_rw_signal(vec![id]).read_only()
    }
}

impl<T: Element + 'static> IntoElement for Rc<T> {
    type V = ReadSignal<Vec<ViewId>>;
    fn into_element(self) -> Self::V {
        let id = self.id();
        RUNTIME.with_borrow_mut({
            let element = self.clone() as Rc<dyn Element>;
            move |r| {
                r.elements
                    .insert(id.node().into(), element);
            }
        });
        create_rw_signal(vec![id]).read_only()
    }
}
impl IntoElement for Fragment {
    type V = Memo<Vec<ViewId>>;
    fn into_element(self) -> Self::V {
        create_memo(move |_| {
            self.children
                .iter()
                .map(|item| item.get())
                .flatten()
                .collect::<Vec<_>>()
        })
    }
}

impl IntoElement for Dynamic {
    type V = ReadSignal<Vec<ViewId>>;
    fn into_element(self) -> Self::V {
        self.signal
    }
}

impl<T: IntoElement + Clone + 'static> IntoElement for ReadSignal<T> {
    type V = Memo<Vec<ViewId>>;

    fn into_element(self) -> Self::V {
        create_memo(move |_| self.get().clone().into_element().get())
    }
}

impl<T: IntoElement + Clone + 'static> IntoElement for Memo<T> {
    type V = Memo<Vec<ViewId>>;
    fn into_element(self) -> Self::V {
        create_memo(move |_| self.get().clone().into_element().get())
    }
}

impl<T: IntoElement + 'static> IntoElement for Vec<T> {
    type V = Memo<Vec<ViewId>>;
    fn into_element(self) -> Self::V {
        let elements = self
            .into_iter()
            .map(|item| item.into_element())
            .collect::<Vec<_>>();
        create_memo(move |_| {
            elements
                .iter()
                .map(|item| item.get())
                .flatten()
                .collect::<Vec<_>>()
        })
    }
}

impl IntoElement for i32 {
    type V = ReadSignal<Vec<ViewId>>;
    fn into_element(self) -> Self::V {
        Text::new(move || self).into_element()
    }
}
