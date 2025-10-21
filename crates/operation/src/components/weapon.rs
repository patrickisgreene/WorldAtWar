use bevy_ecs::prelude::*;
use bevy_asset::prelude::*;
use bevy_internal::prelude::*;

use waw_weapons::data::Weapon;

#[derive(Component, Debug, PartialEq, Clone, Deref, DerefMut)]
pub struct WeaponHandle(pub Handle<Weapon>);

#[derive(Component, Debug, PartialEq, Clone, Deref, DerefMut)]
pub struct WeaponCount(pub usize);