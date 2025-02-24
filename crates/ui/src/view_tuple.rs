use crate::element::{IntoElement, Node};

pub trait ViewTuple {
    fn into_vec(self) -> Node;
}

impl ViewTuple for Node {
    fn into_vec(self) -> Node {
        self
    }
}

impl<VW: IntoElement> ViewTuple for VW {
    fn into_vec(self) -> Node {
        self.into_element()
    }
}

macro_rules! impl_view_tuple {
    ( $( $t:ident),* ; $( $s:tt ),* ) => {
        impl< $( $t: IntoElement, )* > ViewTuple for ( $( $t, )* ) {
            fn into_vec(self)  -> Node  {
                Node::Fragment(vec![$( self.$s.into_element(), )*])
            }
        }
    }
}
pub const VIEW_TUPLE_MAX_ELEMENTS: usize = 8;
impl_view_tuple!(;);
impl_view_tuple!(V0; 0);
impl_view_tuple!(V0, V1; 0, 1);
impl_view_tuple!(V0, V1, V2; 0, 1, 2);
impl_view_tuple!(V0, V1, V2, V3; 0, 1, 2, 3);
impl_view_tuple!(V0, V1, V2, V3, V4; 0, 1, 2, 3, 4);
impl_view_tuple!(V0, V1, V2, V3, V4, V5; 0, 1, 2, 3, 4, 5);
impl_view_tuple!(V0, V1, V2, V3, V4, V5, V6; 0, 1, 2, 3, 4, 5, 6);
impl_view_tuple!(V0, V1, V2, V3, V4, V5, V6, V7; 0, 1, 2, 3, 4, 5, 6, 7);
