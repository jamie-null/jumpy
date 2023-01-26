//! Systems and components related to damage/kill zones

use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PostUpdate, kill_players_in_damage_region);
}

/// A rectangular damage region.
///
/// While this _might_ change in the future, damage regions will kill players immediately, so there
/// is no "damage" field.
#[derive(Debug, Clone, Default, TypeUlid)]
#[ulid = "01GP1X5MBXZNEC4Y0WF5AKCA3Z"]
pub struct DamageRegion {
    /// The size of the damage region in pixels
    pub size: Vec2,
}

impl DamageRegion {
    /// Get the collision rectangle of this damage region, given it's transform.
    pub fn collider_rect(&self, position: Vec3) -> Rect {
        Rect::new(position.x, position.y, self.size.x, self.size.y)
    }
}

/// A component that may be added to a damage region entity to indicate the triggering entity.
///
/// If this entity is a player, it will not be harmed by the damage region.
#[derive(Debug, Clone, TypeUlid)]
#[ulid = "01GP1X4NM7GMEKKZ4FEZ1RK3T0"]
pub struct DamageRegionOwner(pub Entity);

/// A rectangular block region.
///
#[derive(Debug, Clone, Default, TypeUlid)]
#[ulid = "01GQNJ65V1NHZFVXHF1JQD5Z72"]
pub struct BlockRegion {
    /// The size of the damage region in pixels
    pub size: Vec2,
}

impl BlockRegion {
    /// Get the collision rectangle of this damage region, given it's transform.
    pub fn collider_rect(&self, position: Vec3) -> Rect {
        Rect::new(position.x, position.y, self.size.x, self.size.y)
    }
}

/// A component indicating the player protected by a blocking region
///
#[derive(Debug, Clone, TypeUlid)]
#[ulid = "01GQNJHQE35VPDWA6HQY9DE0D4"]
pub struct BlockRegionOwner(pub Entity);

/// System that will eliminate players that are intersecting with a damage region.
fn kill_players_in_damage_region(
    entities: Res<Entities>,
    player_indexes: Comp<PlayerIdx>,
    transforms: Comp<Transform>,
    damage_regions: Comp<DamageRegion>,
    mut camera_shake: ResMut<CameraTraumaEvents>,
    block_regions: Comp<BlockRegion>,
    block_region_owners: Comp<BlockRegionOwner>,
    damage_region_owners: Comp<DamageRegionOwner>,
    bodies: Comp<KinematicBody>,
    mut player_events: ResMut<PlayerEvents>,
) {
    for (player_ent, (_idx, transform, body)) in
        entities.iter_with((&player_indexes, &transforms, &bodies))
    {
        let player_rect = body.collider_rect(transform.translation);
        for (ent, (damage_region, transform)) in entities.iter_with((&damage_regions, &transforms))
        {
            let owner = damage_region_owners.get(ent);
            // Don't damage the player that owns this damage region
            if let Some(owner) = owner {
                if owner.0 == player_ent {
                    continue;
                }
            }

            let damage_rect = damage_region.collider_rect(transform.translation);
            if player_rect.overlaps(&damage_rect) {
                let mut kill = true;

                for (blk, (block_region, transform)) in
                    entities.iter_with((&block_regions, &transforms))
                {
                    let blk_owner = block_region_owners.get(blk);
                    if let Some(blk_owner) = blk_owner {
                        if blk_owner.0 == player_ent {
                            let block_rect = block_region.collider_rect(transform.translation);
                            if block_rect.overlaps(&damage_rect)
                                && player_rect.overlaps(&block_rect)
                            {
                                kill = false;
                                camera_shake.send(0.5);
                                break;
                            }
                        }
                    }
                }
                if kill {
                    player_events.kill(player_ent);
                }
            }
        }
    }
}
