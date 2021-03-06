use crate::components::{InputComponent, MovementComponent, PositionComponent, SpriteComponent};
use crate::debug_log::*;
use crate::map::Map;
use crate::shared_types::{Coordinate, Velocity, ZERO_VELOCITY};

use fixed::traits::FromFixed;

use core::convert::TryInto;

/// Maximum player speed, in pixels per frame
/// A value of 1 means the player can move at most 60 pixels or 7.5 tiles a second.
const PLAYER_MAX_VELOCITY: Velocity =
    Velocity::from_bits(0b0000_0000_0000_0000_0000_0001_0000_0000); // 1
const PLAYER_MIN_VELOCITY: Velocity =
    Velocity::from_bits(-0b0000_0000_0000_0000_0000_0001_0000_0001); // -1

/// How much the player's speed changes for each frame the button is held down, in pixels per second.
const VELOCITY_DELTA_PER_FRAME: Velocity =
    Velocity::from_bits(0b0_0000_0000_0000_0000_0000_0000_1101); // 0.1

use tiny_ecs::{ECSError, Entities};
/// An ECS system which moves entity sprites based on their velocity
pub struct MovementSystem {}

impl MovementSystem {
    /// For each entity that is live,
    /// check whether it has a sprite and move it if it does.
    ///
    /// If the camera should stay focused on the entity, move the map instead of the entity.
    pub fn tick(
        ecs: &mut Entities,
        live_entities: &[usize],
        map: &mut Map,
    ) -> Result<(), ECSError> {
        let mut movables = ecs.borrow_mut::<MovementComponent>().unwrap();
        let inputables = ecs.borrow_mut::<InputComponent>().unwrap();
        let mut positionables = ecs.borrow_mut::<PositionComponent>().unwrap();
        let mut sprites = ecs.borrow_mut::<SpriteComponent>().unwrap();
        for id in live_entities {
            let id = *id;
            if ecs.entity_contains::<MovementComponent>(id) {
                let e_movement: &mut MovementComponent = movables.get_mut(id).unwrap();
                // Process position updates caused by input
                if ecs.entity_contains::<InputComponent>(id) {
                    let e_input: &InputComponent = inputables.get(id).unwrap();
                    update_movement_based_on_input(e_input, e_movement);
                }

                // Process updates to entity positions
                if ecs.entity_contains::<PositionComponent>(id) {
                    let e_position: &mut PositionComponent = positionables.get_mut(id).unwrap();

                    // Entities which the camera is centered on are only allowed to move if it
                    // wouldn't cause the rendered background area to go OOB
                    let mut entity_moved: bool = true;
                    if e_movement.keep_camera_centered_on
                        && (e_movement.pending_movement_delta_x != ZERO_VELOCITY
                            || e_movement.pending_movement_delta_y != ZERO_VELOCITY)
                    {
                        let (map_delta_x, map_delta_y) = e_movement.get_pending_movement_delta();
                        if map.can_scroll(map_delta_x, map_delta_y) {
                            map.try_scroll(map_delta_x, map_delta_y);
                        } else {
                            entity_moved = false;
                        }
                    }
                    if entity_moved {
                        update_position_based_on_movement(e_movement, e_position);
                    }

                    // Get rid of processed movement delta
                    e_movement.reset_pending_movement_delta();
                }
            }
            // Process updates to entity sprites caused by position change
            if ecs.entity_contains::<SpriteComponent>(id)
                && ecs.entity_contains::<PositionComponent>(id)
            {
                // Do not move sprites that the camera is centered on
                if ecs.entity_contains::<MovementComponent>(id) {
                    let e_movement: &mut MovementComponent = movables.get_mut(id).unwrap();
                    if e_movement.keep_camera_centered_on {
                        // Skip this entity
                        continue;
                    }
                }
                let e_sprite: &mut SpriteComponent = sprites.get_mut(id).unwrap();
                let e_position: &mut PositionComponent = positionables.get_mut(id).unwrap();
                update_sprite_based_on_position(map, e_position, e_sprite);
            }
        }
        return Ok(());
    }
}

/// Update an entity's position based on it's input component.
fn update_movement_based_on_input(ic: &InputComponent, mc: &mut MovementComponent) {
    // If the button is pressed, accelerate
    if ic.left_pressed {
        if mc.x_velocity > PLAYER_MIN_VELOCITY {
            mc.x_velocity -= VELOCITY_DELTA_PER_FRAME;
        }
        mc.pending_movement_delta_x += mc.x_velocity;
    // If the button isn't pressed and we aren't moving in the opposite direction, decelerate
    } else if !ic.left_pressed && !ic.right_pressed && mc.x_velocity < ZERO_VELOCITY {
        if mc.x_velocity < -VELOCITY_DELTA_PER_FRAME {
            mc.x_velocity += VELOCITY_DELTA_PER_FRAME;
        // Make sure we don't overshoot and cause a drift into positive X velocity
        } else {
            mc.x_velocity = ZERO_VELOCITY;
        }
        mc.pending_movement_delta_x += mc.x_velocity;
    }

    if ic.right_pressed {
        if mc.x_velocity < PLAYER_MAX_VELOCITY {
            mc.x_velocity += VELOCITY_DELTA_PER_FRAME;
        }
        mc.pending_movement_delta_x += mc.x_velocity;
    } else if !ic.right_pressed && !ic.left_pressed && mc.x_velocity > ZERO_VELOCITY {
        if mc.x_velocity > VELOCITY_DELTA_PER_FRAME {
            mc.x_velocity -= VELOCITY_DELTA_PER_FRAME;
        } else {
            mc.x_velocity = ZERO_VELOCITY;
        }
        mc.pending_movement_delta_x += mc.x_velocity;
    }

    // If no buttons causing movement on the X axis are pressed, decelerate towards 0

    if ic.up_pressed {
        if mc.y_velocity > PLAYER_MIN_VELOCITY {
            mc.y_velocity -= VELOCITY_DELTA_PER_FRAME;
        }
        mc.pending_movement_delta_y += mc.y_velocity;
    } else if !ic.up_pressed && !ic.down_pressed && mc.y_velocity < ZERO_VELOCITY {
        if mc.y_velocity < -VELOCITY_DELTA_PER_FRAME {
            mc.y_velocity += VELOCITY_DELTA_PER_FRAME;
        } else {
            mc.y_velocity = ZERO_VELOCITY;
        }
        mc.pending_movement_delta_y += mc.y_velocity;
    }

    if ic.down_pressed {
        if mc.y_velocity < PLAYER_MAX_VELOCITY {
            mc.y_velocity += VELOCITY_DELTA_PER_FRAME;
        }
        mc.pending_movement_delta_y += mc.y_velocity;
    } else if !ic.down_pressed && !ic.up_pressed && mc.y_velocity > ZERO_VELOCITY {
        if mc.y_velocity > VELOCITY_DELTA_PER_FRAME {
            mc.y_velocity -= VELOCITY_DELTA_PER_FRAME;
        } else {
            mc.y_velocity = ZERO_VELOCITY;
        }
        mc.pending_movement_delta_y += mc.y_velocity;
    }
}

/// This function updates the position of entities based on their pending_movement fields.
fn update_position_based_on_movement(mc: &MovementComponent, pc: &mut PositionComponent) {
    if mc.pending_movement_delta_x > ZERO_VELOCITY || mc.pending_movement_delta_y > ZERO_VELOCITY {
        // Add the pending movement to entity's position
        (pc.0).0 += Coordinate::from_fixed(mc.pending_movement_delta_x);
        (pc.0).1 += Coordinate::from_fixed(mc.pending_movement_delta_y);
    }
}

// Updates the sprite's relative onscreen position based on changes in it's absolute map coordinates
fn update_sprite_based_on_position(map: &Map, pc: &PositionComponent, sp: &mut SpriteComponent) {
    // Check whether sprite would be visible on screen (if not, disable drawing)
    let (sprite_top_left_x, sprite_top_left_y) = pc.floor();
    let sh = sp.get_handle();
    let (x_size, y_size) = sh.sprite_size.to_size_in_px();
    let sprite_bottom_right_x = sprite_top_left_x + (x_size as u32);
    let sprite_bottom_right_y = sprite_top_left_y + (y_size as u32);
    if !map.is_area_visible(
        sprite_top_left_x,
        sprite_top_left_y,
        sprite_bottom_right_x,
        sprite_bottom_right_y,
    ) {
        // TODO: Temporarily eject sprite from OAM to make room for visible ones
        debug_log!(
            Subsystems::MovementSystem,
            "Sprite now offscreen, making invisible"
        );
        sh.set_visibility(false);
    } else {
        if !sh.get_visibility() {
            debug_log!(
                Subsystems::MovementSystem,
                "Sprite now onscreen, making visible again"
            );
            sh.set_visibility(true);
        }
        let (map_top_left_x, map_top_left_y) = map.get_top_left_corner_coords();
        // Convert the map coordinates to coordinates relative to the top-left corner of the screen
        // (which are the ones the hardware cares about)
        let onscreen_x: i16 = ((sprite_top_left_x as i32) - (map_top_left_x as i32))
            .try_into()
            .unwrap();
        let onscreen_y: i16 = ((sprite_top_left_y as i32) - (map_top_left_y as i32))
            .try_into()
            .unwrap();
        // Because of the way the GBA stores sprite coordinates, we have to do some funkiness for negative values to work.
        let onscreen_x: u16 = if onscreen_x > 0 {
            onscreen_x.try_into().unwrap()
        } else {
            (onscreen_x & 0x01FF).try_into().unwrap()
        };
        let onscreen_y: u16 = if onscreen_y > 0 {
            onscreen_y.try_into().unwrap()
        } else {
            (onscreen_y & 0x00FF).try_into().unwrap()
        };
        // Actually move the sprite
        debug_log!(
            Subsystems::MovementSystem,
            "Moving sprite to onscreen coords {} {}",
            onscreen_x,
            onscreen_y
        );
        sh.set_x_pos(onscreen_x);
        sh.set_y_pos(onscreen_y);
    }
}
