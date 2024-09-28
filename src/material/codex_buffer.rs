use crate::assets::AssetState;
use bevy::{
    prelude::*,
    render::{
        render_resource::{encase::internal::WriteInto, Buffer, ShaderSize, StorageBuffer},
        renderer::{RenderDevice, RenderQueue},
    },
};
use expl_codex::{Codex, Id};
use std::marker::PhantomData;

pub trait CodexBufferValue:
    Default + ShaderSize + WriteInto + for<'v> std::convert::From<&'v Self::CodexValue>
{
    type CodexValue: Default + TypePath;
}

#[derive(Default)]
pub struct CodexBufferPlugin<V: CodexBufferValue> {
    _phantom_data: PhantomData<V>,
}

impl<V> Plugin for CodexBufferPlugin<V>
where
    V: CodexBufferValue + 'static + Send + Sync,
    V::CodexValue: 'static + Send + Sync + std::fmt::Debug,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<CodexBuffer<V>>()
            .add_systems(OnEnter(AssetState::Loaded), prepare_buffer::<V>);
    }
}

#[derive(Resource, Default)]
pub struct CodexBuffer<V>
where
    V: CodexBufferValue,
{
    keys: Vec<Id<V::CodexValue>>,
    data: StorageBuffer<Vec<V>>,
}

impl<V> CodexBuffer<V>
where
    V: CodexBufferValue,
{
    pub fn as_index(&self, id: &Id<V::CodexValue>) -> Option<usize> {
        self.keys.iter().position(|&key| key == *id)
    }

    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.data.write_buffer(render_device, render_queue);
    }

    pub fn buffer(&self) -> Option<&Buffer> {
        self.data.buffer()
    }
}

impl<'a, V> Extend<(&'a Id<V::CodexValue>, &'a V::CodexValue)> for CodexBuffer<V>
where
    V: CodexBufferValue,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (&'a Id<V::CodexValue>, &'a V::CodexValue)>,
    {
        for (id, value) in iter {
            if let Some(idx) = self.as_index(id) {
                self.data.get_mut()[idx] = value.into();
            } else {
                self.keys.push(*id);
                self.data.get_mut().push(value.into());
            }
        }
    }
}

fn prepare_buffer<V>(
    mut buffer: ResMut<CodexBuffer<V>>,
    codex_assets: Res<Assets<Codex<V::CodexValue>>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) where
    V: CodexBufferValue + 'static + Send + Sync,
    V::CodexValue: Send + Sync + std::fmt::Debug,
{
    for (_, codex) in codex_assets.iter() {
        buffer.extend(codex.iter());
    }
    buffer.write_buffer(&render_device, &render_queue);
}
