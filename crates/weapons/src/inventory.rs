use bevy_ecs::prelude::*;
use bevy_asset::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_platform::collections::HashMap;

use crate::data::Weapon;

#[derive(Component, Default, Debug, PartialEq, Clone, Deref, DerefMut)]
pub struct Inventory {
    inner: HashMap<Handle<Weapon>, usize>
}