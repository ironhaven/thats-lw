use rand::distributions::Bernoulli;
use rand::distributions::Uniform;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use std::vec::Vec;

/*
Weapon          Hit Chance % 	Fire Rate 	Damage 	Armor Pen.
Single Plasma 	18/33/48 	    1.15 	    450 	0%
Double Plasma 	25/40/55 	    1.25 	    800 	20%
Fusion Lance 	30/45/60 	    1.25 	    1300 	50%

Avalanche       40              2.0         340     0
Stingray        40              1.5         200     50


Ship            Health  Armor  Pen
Scout           750     0      15
Fighter         850     60     35
Intercepter     2500    25     0
Assault Carrier
 */

struct Tech {
    flags: u32,
    alien_research: u32,
}

struct BattleSetup {
    xcom_start_health: f64,
    xcom_effective_damage: Uniform<i32>,
    xcom_hit_chance: Bernoulli,
    xcom_crit_chance: Bernoulli,
    xcom_fire_rate: i32,
    ufo_start_health: f64,
    ufo_effective_damage: Uniform<i32>,
    ufo_hit_chance: Bernoulli,
    ufo_crit_chance: Bernoulli,
    ufo_fire_rate: i32,
}

impl BattleSetup {
    fn calculate(xcom: Ship, ufo: Ship, stance: Stance, research: ()) -> Self {
        BattleSetup {
            xcom_start_health: xcom.health,
            // Damage * Mitigation * kills * research bonus
            xcom_effective_damage:  {
                let min = dbg!(xcom.weapon.damange as f64) * (1.0 - ufo.incoming_mitigation(&xcom)) * (1.0 + (xcom.kills as f64 / 100.0));
                let max = dbg!(min * 1.5);
                
                Uniform::new(min.round() as i32, max.round() as i32)
            },
            // Weapon hit chance  * research
            xcom_hit_chance:xcom.outward_hit_chance(stance),
            // (xcom pen - ufo armor) / 2
            xcom_crit_chance: xcom.outward_crit_chance(&ufo),
            xcom_fire_rate: xcom.weapon.rate,
            ufo_start_health: ufo.health,
            // Damage * Mitigation * research bonus
            ufo_effective_damage: {
                let min = ufo.weapon.damange as f64 * (1.0 - xcom.incoming_mitigation(&ufo));
                let max = min * 1.5;
                Uniform::new(min.round() as i32, max.round() as i32)
            },
            ufo_hit_chance: ufo.outward_hit_chance(stance),
            ufo_crit_chance: ufo.outward_crit_chance(&xcom),
            ufo_fire_rate: ufo.weapon.rate,
        }
    }
    fn run(&self, rng: &mut impl Rng, xcom_percent: f64, ufo_percent: f64) -> (f64, f64) {
        let mut xcom_health = (self.xcom_start_health * xcom_percent) as i32;
        let mut ufo_health = (self.ufo_start_health * ufo_percent) as i32;
        // TODO: random start time
        let mut ufo_next = self.ufo_fire_rate;
        let mut xcom_next = self.xcom_fire_rate;
        let mut time = 0;

        
        // TODO: early kill/abort percentages
        // TODO: abort time
        while time < 10_000 && ufo_health > 0 && xcom_health > 0 {
            println!("{time}");
            if ufo_next <= xcom_next {
                time = ufo_next;
                if rng.sample(self.ufo_hit_chance) {
                    let mut d = rng.sample(self.ufo_effective_damage);
                    if rng.sample(self.ufo_crit_chance) {
                        d *= 2;
                    };
                    xcom_health -= d;
                    println!("ufo hit xcom for {d}");
                } else {
                    println!("ufo missed");
                }
                // TODO: projectile travel time
                ufo_next += self.ufo_fire_rate;
            } else {
                time = xcom_next;
                if rng.sample(self.xcom_hit_chance) {
                    let mut d = rng.sample(self.xcom_effective_damage);
                    if rng.sample(self.xcom_crit_chance) {
                        d *= 2;
                    };
                    ufo_health -= d;
                    println!("xcom hit ufo for {d}");
                } else {
                    println!("xcom missed");
                }

                xcom_next += self.xcom_fire_rate;
            }
            }
        // FEAT: return extra data?
        (xcom_health as f64 / self.xcom_start_health, ufo_health as f64 / self.xcom_start_health)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Stance {
    Def,
    Bal,
    Agg,
}
#[derive(Clone)]
struct Ship {
    health: f64,
    armor: u32,
    pen: u32,
    kills: u32,
    weapon: Weapon,
}

impl Ship {
    fn pen(&self) -> u32 {
        self.pen + self.weapon.pen
    }
    fn outward_hit_chance(&self, stance: Stance) -> Bernoulli {
        Bernoulli::from_ratio(
            dbg!(self.weapon.hit_chance(stance) + (self.kills * 3).min(30)),
            100,
        )
        .unwrap()
    }
    fn outward_crit_chance(&self, other: &Ship) -> Bernoulli {
        // ((pen - armor) / 2) / 100 -> (pen - armor) / 200
        Bernoulli::from_ratio(self.pen().saturating_sub(other.armor).max(10).min(50), 200).unwrap()
    }
    fn incoming_mitigation(&self, other: &Ship) -> f64 {
        self.armor.saturating_sub(other.pen()).min(95) as f64 / 100.0
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Weapon {
    hit_chance: u32,
    rate: i32,
    damange: i32,
    pen: u32,
}

impl Weapon {
    fn hit_chance(&self, stance: Stance) -> u32 {
        match stance {
            Stance::Def => self.hit_chance.saturating_sub(15),
            Stance::Bal => self.hit_chance,
            Stance::Agg => self.hit_chance + 15,
        }
    }
}
fn main() {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

    let avalanche = Weapon {
        hit_chance: 40,
        rate: 2000,
        damange: 340,
        pen: 0,
    };
    let single_plasma = Weapon {
        hit_chance: 33,
        rate: 1150,
        damange: 450,
        pen: 0,
    };
    let ufo = Ship {
        health: 750.0,
        armor: 0,
        pen: 15,
        kills: 0,
        weapon: single_plasma,
    };
    let xcom = Ship {
        health: 2500.0,
        armor: 25,
        pen: 0,
        kills: 0,
        weapon: avalanche,
    };

    let stance = Stance::Agg;

    let battle = BattleSetup::calculate(xcom.clone(), ufo.clone(), stance, ());
    let mut xcom_health = battle.xcom_start_health as i32;
    let mut ufo_health = battle.ufo_start_health as i32;
    let mut ufo_next = ufo.weapon.rate;
    let mut xcom_next = xcom.weapon.rate;
    let mut time = 0;

    println!(
        "{} {}",
        ufo.weapon.hit_chance(stance) as f64 / 100.0,
        xcom.weapon.hit_chance(stance) as f64 / 100.0
    );

    let (xcom, ufo) = dbg!(battle.run(&mut rng, 1.0, 1.0));
    let (xcom, ufo) = dbg!(battle.run(&mut rng, xcom, ufo));
    let (xcom, ufo) = dbg!(battle.run(&mut rng, xcom, ufo));
    println!("Hello, world!");

    //xcom
}
