use bevy::prelude::*;

use crate::entities::particle::components::ParticleElement;
use crate::entities::particle::resources::convert_to_grid_position;
use crate::entities::particle::systems::*;

use crate::{
    entities::particle::resources::ParticleMap,
    entities::particle::components::ParticleTagQuickSand,
    entities::player::components::Player,
    entities::enemy::components::Enemy,
    entities::spaceship::components::Spaceship,
};



//#[derive(Component)]
//pub struct DoNotSearchCollide;

#[derive(Component, Clone, Debug)]
pub struct Hitbox {
    pub width: f32,
    pub height: f32,
    pub offset: Vec2, //center of the hitbox
}

impl Hitbox {
    pub fn new(width: f32, height: f32, offset: Vec2) -> Self {
        Self {
            width,
            height,
            offset,
        }
    }
    pub fn collides_with(&self, other: &Hitbox) -> bool {
        //tr = topright corner
        let self_tr = self.offset + Vec2::new(self.width,self.height)/2.0;
        let self_bl = self.offset - Vec2::new(self.width,self.height)/2.0;
        let other_bl = other.offset - Vec2::new(other.width,other.height)/2.0;
        let other_tr = other.offset + Vec2::new(other.width,other.height)/2.0;
        self_tr.x > other_bl.x && self_bl.x < other_tr.x && self_tr.y > other_bl.y && self_bl.y < other_tr.y
    }
    pub fn all_player_collisions(&self, hitboxes: &Query<&Hitbox, Without<Player>>)  -> bool {
        for hitbox in hitboxes.iter() {
            if self.collides_with(hitbox) {
                //info!("Collision detected between {:?} and {:?}", self, hitbox);
                return true;
            }
        }
        false
    }
    pub fn player_enemy_collision(&self, hitboxes: &Query<&Hitbox, (With<Enemy>, Without<Player>)>)  -> bool {
        for hitbox in hitboxes.iter() {
            if self.collides_with(hitbox) {
                return true;
            }
        }
        false
    }
    pub fn all_enemy_collisions(&self, hitboxes: &Query<&Hitbox, Without<Enemy>>)  -> bool {
        for hitbox in hitboxes.iter() {
            if self.collides_with(hitbox) {
                //info!("Enemy Collision detected between {:?} and {:?}", self, hitbox);
                return true;
            }
        }
        false
    }
    pub fn all_ship_collisions(&self, hitboxes: &Query<&Hitbox, Without<Spaceship>>)  -> bool {
        for hitbox in hitboxes.iter() {
            if self.collides_with(hitbox) {
                return true;
            }
        }
        false
    }
    /*pub fn contains(&self, position: &Vec2) -> bool {
        // 假设 hitbox 以中心为原点
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;
        position.x >= self.offset.x - half_width &&
        position.x <= self.offset.x + half_width &&
        position.y >= self.offset.y - half_height &&
        position.y <= self.offset.y + half_height
    }

    pub fn are_all_grid_tiles_air(&self, map: &ResMut<ParticleMap>) -> bool {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        for x in top_left_x..=bottom_right_x {
            for y in bottom_right_y..=top_left_y {
                if map.get(x, y) != ParticleData::Air {
                    return false;
                }
            }   
        }
        true
    }*/

    pub fn are_any_grid_tiles_water(&self, map: &ResMut<ParticleMap>) -> bool {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        for x in top_left_x..=bottom_right_x {
            for y in bottom_right_y..=top_left_y {
                if map.get_element_at((x, y)) == ParticleElement::Water {
                    return true;
                }
            }   
        }
        false
    }

    pub fn ratio_of_water_grid_tiles(&self, map: &ResMut<ParticleMap>) -> f32 {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        let mut count = 0;
        let mut no_count = 0;
        for x in top_left_x..=bottom_right_x {
            for y in bottom_right_y..=top_left_y {
                if map.get_element_at((x, y)) == ParticleElement::Water {
                    count+=1;
                }
                else {
                    no_count+=1;
                }
            }   
        }
        count as f32 / (count + no_count) as f32
    }

    pub fn ratio_of_lava_grid_tiles(&self, map: &ResMut<ParticleMap>) -> f32 {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        let mut count = 0;
        let mut no_count = 0;
        for x in top_left_x..=bottom_right_x {
            for y in bottom_right_y..=top_left_y {
                if map.get_element_at((x, y)) == ParticleElement::Lava {
                    count+=1;
                }
                else {
                    no_count+=1;
                }
            }   
        }
        count as f32 / (count + no_count) as f32
    }
  
    pub fn ratio_of_quicksand_grid_tiles(&self, map: &ResMut<ParticleMap>) -> f32 {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        let mut count = 0;
        let mut no_count = 0;
        for x in top_left_x..=bottom_right_x {
            for y in bottom_right_y..=top_left_y {
                if map.get_element_at((x, y)) == ParticleElement::QuickSand {
                    count+=1;
                }
                else {
                    no_count+=1;
                }
            }   
        }
        count as f32 / (count + no_count) as f32
    }
    pub fn ratio_of_slime_grid_tiles(&self, map: &ResMut<ParticleMap>) -> f32 {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        let mut count = 0;
        let mut no_count = 0;
        for x in top_left_x..=bottom_right_x {
            for y in bottom_right_y..=top_left_y {
                if map.get_element_at((x, y)) == ParticleElement::Slime {
                    count+=1;
                }
                else {
                    no_count+=1;
                }
            }   
        }
        count as f32 / (count + no_count) as f32
    }

    pub fn ratio_of_healing_spring_grid_tiles(&self, map: &ResMut<ParticleMap>) -> f32 {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        let mut count = 0;
        let mut no_count = 0;
        for x in top_left_x..=bottom_right_x {
            for y in bottom_right_y..=top_left_y {
                if map.get_element_at((x, y)) == ParticleElement::Healing_Spring {
                    count+=1;
                }
                else {
                    no_count+=1;
                }
            }   
        }
        count as f32 / (count + no_count) as f32
    }
      
      
      
      
    pub fn ratio_of_toxic_gas_tiles(&self, map: &ResMut<ParticleMap>) -> f32 {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        let mut count = 0;
        let mut no_count = 0;
        for x in top_left_x..=bottom_right_x {
            for y in bottom_right_y..=top_left_y {
                if map.get_element_at((x, y)) == ParticleElement::ToxicGas {
                    count+=1;
                }
                else {
                    no_count+=1;
                }
            }   
        }
        count as f32 / (count + no_count) as f32
    }
    
    // return the grid position of the top left and bottom right corners of the hitbox
    // (top_left_x, top_left_y, bottom_right_x, bottom_right_y) 
    pub fn get_grid_tiles_to_check(&self) -> (i32, i32, i32, i32) { //
        let top_left_grid_pos = convert_to_grid_position(self.offset.x - self.width / 2.0, self.offset.y + self.height / 2.0);
        let bottom_right_grid_pos = convert_to_grid_position(self.offset.x + self.width / 2.0, self.offset.y - self.height / 2.0);
        (top_left_grid_pos.0, top_left_grid_pos.1, bottom_right_grid_pos.0, bottom_right_grid_pos.1)
    }

    pub fn number_of_water_grid_tiles_colliding(&self, map: &ResMut<ParticleMap>) -> i32 {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        let mut count = 0;
        for x in top_left_x..=bottom_right_x {
            for y in bottom_right_y..=top_left_y {
                if map.get_element_at((x, y)) == ParticleElement::Water {
                    count+=1;
                }
            }   
        }
        count
    }

    pub fn is_particle_in_hitbox(&self, (x,y): (i32, i32)) -> bool {
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = self.get_grid_tiles_to_check();
        x >= top_left_x && x <= bottom_right_x && y <= top_left_y && y >= bottom_right_y
    }
}