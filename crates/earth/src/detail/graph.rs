use bevy_ecs::prelude::*;

/// The entity that this entity is targeting.
///
/// This is the source of truth for the relationship,
/// and can be modified directly to change the target.
#[derive(Component, Debug)]
#[relationship(relationship_target = Branch)]
pub struct Leaf(pub Entity);

/// All entities that are targeting this entity.
///
/// This component is updated reactively using the component hooks introduced by deriving
/// the [`Relationship`] trait. We should not modify this component directly,
/// but can safely read its field. In a larger project, we could enforce this through the use of
/// private fields and public getters.
#[derive(Component, Debug)]
#[relationship_target(relationship = Leaf)]
pub struct Branch(Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = UrbanEntityFor)]
pub struct UrbanEntity(Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = UrbanEntity)]
pub struct UrbanEntityFor(Entity);