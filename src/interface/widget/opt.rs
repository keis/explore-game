use bevy_quill_core::{Cond, Cx, View, ViewTemplate};

#[derive(Clone, PartialEq)]
struct Unwrap<T>(Option<T>)
where
    T: ViewTemplate + Clone + PartialEq;

impl<T> ViewTemplate for Unwrap<T>
where
    T: ViewTemplate + Clone + PartialEq,
{
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        self.0.as_ref().unwrap().create(cx)
    }
}

#[derive(Clone, PartialEq)]
pub struct Opt<T>
where
    T: ViewTemplate + Clone + PartialEq,
{
    inner: Option<T>,
}

impl<T> Opt<T>
where
    T: ViewTemplate + Clone + PartialEq,
{
    pub fn new(inner: Option<T>) -> Self {
        Self { inner }
    }
}

impl<T> ViewTemplate for Opt<T>
where
    T: ViewTemplate + Clone + PartialEq,
{
    type View = impl View;

    fn create(&self, _cx: &mut Cx) -> Self::View {
        let inner = self.inner.clone();
        Cond::new(inner.is_some(), Unwrap(inner), ())
    }
}
