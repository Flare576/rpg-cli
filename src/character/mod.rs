use serde::{Deserialize, Serialize};
use std::cmp::max;

mod class;
use crate::randomizer::Randomizer;
use class::Class;

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    class: Class,
    pub name: String,

    pub level: i32,
    pub xp: i32,

    pub max_hp: i32,
    pub current_hp: i32,

    pub strength: i32,
    pub speed: i32,
}

impl Character {
    pub fn player() -> Self {
        Self::new(Class::Hero, "hero", 1)
    }

    pub fn enemy(level: i32) -> Self {
        Self::new(Class::Enemy, "enemy", level)
    }

    pub fn is_player(&self) -> bool {
        matches!(self.class, Class::Hero)
    }

    fn new(class: Class, name: &str, level: i32) -> Self {
        let params = class.params();
        let mut character = Self {
            class,
            level: 1,
            name: String::from(name),
            xp: 0,
            max_hp: params.start_hp,
            current_hp: params.start_hp,
            strength: params.start_strength,
            speed: params.start_speed,
        };

        for _ in 1..level {
            character.increase_level();
        }

        character
    }

    /// Raise the level and all the character stats.
    fn increase_level(&mut self) {
        let params = self.class.params();

        self.level += 1;
        self.strength = Randomizer::stat(self.strength, params.strength_rate);
        self.speed = Randomizer::stat(self.speed, params.speed_rate);

        // the current should increase proportionally but not
        // erase previous damage
        let previous_damage = self.max_hp - self.current_hp;
        self.max_hp = Randomizer::stat(self.max_hp, params.hp_rate);
        self.current_hp = self.max_hp - previous_damage;
    }

    /// Add to the accumulated experience points, possibly increasing the level.
    pub fn add_experience(&mut self, xp: i32) -> bool {
        self.xp += xp;
        let for_next = self.xp_for_next();
        if self.xp >= for_next {
            self.increase_level();
            self.xp -= for_next;
            return true;
        }
        false
    }

    pub fn receive_damage(&mut self, damage: i32) {
        self.current_hp = max(0, self.current_hp - damage);
    }

    pub fn is_dead(&self) -> bool {
        self.current_hp == 0
    }

    pub fn heal(&mut self) -> i32 {
        let recovered = self.max_hp - self.current_hp;
        self.current_hp = self.max_hp;
        recovered
    }

    /// How many experience points are required to move to the next level.
    pub fn xp_for_next(&self) -> i32 {
        let exp = 1.5;
        let base_xp = 30.0;
        (base_xp * (self.level as f64).powf(exp)) as i32
    }

    /// Generate a randomized damage numer based on the attacker strength
    /// and the receiver strength.
    pub fn damage(&self, receiver: &Self) -> i32 {
        // Possible improvements: use different attack and defense stats,
        // incorporate weapon and armor effect.

        let str_10 = self.strength as f64 * 0.1;

        // attenuate the level based difference to help the weaker player
        let level_diff_effect = if self.level < receiver.level {
            (self.level - receiver.level) as f64 * str_10
        } else {
            (self.level - receiver.level) as f64 / 2.0 * str_10
        };

        let damage = (self.strength as f64 + level_diff_effect) as i32;
        max(str_10.ceil() as i32, Randomizer::damage(damage))
    }

    /// How many experience points are gained by inflicting damage to an enemy.
    pub fn xp_gained(&self, receiver: &Self, damage: i32) -> i32 {
        // should the player also gain experience by damage received?

        if receiver.level > self.level {
            damage * (1 + receiver.level - self.level)
        } else {
            damage / (1 + self.level - receiver.level)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_char() -> Character {
        Character::new(Class::Test, "hero", 1)
    }

    #[test]
    fn test_new() {
        let hero = new_char();

        assert_eq!(1, hero.level);
        assert_eq!(0, hero.xp);

        let params = Class::Test.params();
        assert_eq!(params.start_hp, hero.current_hp);
        assert_eq!(params.start_hp, hero.max_hp);
        assert_eq!(params.start_strength, hero.strength);
        assert_eq!(params.start_speed, hero.speed);
    }

    #[test]
    fn test_increase_level() {
        let mut hero = new_char();

        let params = Class::Test.params();
        // assert what we're assuming are the params in the rest of the test
        assert_eq!(0.3, params.hp_rate);
        assert_eq!(0.1, params.strength_rate);
        assert_eq!(0.1, params.speed_rate);

        hero.max_hp = 20;
        hero.current_hp = 20;
        hero.strength = 10;
        hero.speed = 5;

        hero.increase_level();
        assert_eq!(2, hero.level);
        assert_eq!(26, hero.max_hp);
        assert_eq!(11, hero.strength);
        assert_eq!(6, hero.speed);

        let damage = 7;
        hero.current_hp -= damage;

        hero.increase_level();
        assert_eq!(3, hero.level);
        assert_eq!(hero.current_hp, hero.max_hp - damage);
    }

    #[test]
    fn test_damage() {
        let mut hero = new_char();
        let mut foe = new_char();

        // 1 vs 1 -- no level-based effect
        hero.strength = 10;
        foe.strength = 10;
        assert_eq!(10, hero.damage(&foe));

        // level 1 vs level 2
        foe.level = 2;
        foe.strength = 15;
        assert_eq!(9, hero.damage(&foe));

        // level 2 vs level 1
        assert_eq!(15, foe.damage(&hero));

        // level 1 vs level 5
        foe.level = 5;
        foe.strength = 40;
        assert_eq!(6, hero.damage(&foe));

        // level 5 vs level 1
        assert_eq!(48, foe.damage(&hero));
    }

    #[test]
    fn test_xp_gained() {
        let hero = new_char();
        let mut foe = new_char();
        let damage = 10;

        // 1 vs 1 -- no level-based effect
        let xp = hero.xp_gained(&foe, damage);
        assert_eq!(damage, xp);

        // level 1 vs level 2
        foe.level = 2;
        let xp = hero.xp_gained(&foe, damage);
        assert_eq!(2 * damage, xp);

        // level 2 vs level 1
        let xp = foe.xp_gained(&hero, damage);
        assert_eq!(damage / 2, xp);

        // level 1 vs level 5
        foe.level = 5;
        let xp = hero.xp_gained(&foe, damage);
        assert_eq!(5 * damage, xp);

        // level 5 vs level 1
        let xp = foe.xp_gained(&hero, damage);
        assert_eq!(damage / 5, xp);
    }

    #[test]
    fn test_xp_for_next() {
        let mut hero = new_char();
        assert_eq!(30, hero.xp_for_next());
        hero.increase_level();
        assert_eq!(84, hero.xp_for_next());
        hero.increase_level();
        assert_eq!(155, hero.xp_for_next());
    }

    #[test]
    fn test_add_experience() {
        let mut hero = new_char();
        assert_eq!(1, hero.level);
        assert_eq!(0, hero.xp);

        let level_up = hero.add_experience(20);
        assert!(!level_up);
        assert_eq!(1, hero.level);
        assert_eq!(20, hero.xp);

        let level_up = hero.add_experience(25);
        assert!(level_up);
        assert_eq!(2, hero.level);
        assert_eq!(15, hero.xp);
    }
}