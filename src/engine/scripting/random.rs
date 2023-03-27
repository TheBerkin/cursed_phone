use std::{cmp, collections::HashSet};

use crate::engine::*;
use perlin2d::PerlinNoise2D;
use rand::{SeedableRng, distributions::Uniform, prelude::Distribution, RngCore};
use rand_xoshiro::Xoshiro256PlusPlus;

use super::lua_error;

pub struct LuaRandom(Xoshiro256PlusPlus);

impl LuaRandom {
    pub fn new() -> Self {
        Self::with_seed_u64(rand::thread_rng().next_u64())
    }

    pub fn with_seed_u64(seed: u64) -> Self {
        Self(Xoshiro256PlusPlus::seed_from_u64(seed))
    }

    pub fn with_seed_i64(seed: i64) -> Self {
        Self(Xoshiro256PlusPlus::seed_from_u64(u64::from_le_bytes(seed.to_le_bytes())))
    }
}

impl LuaUserData for LuaRandom {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("bits_32", |_, this, ()| {
            Ok(this.0.gen::<i32>())
        });
        methods.add_method_mut("bits_64", |_, this, ()| {
            Ok(this.0.gen::<i64>())
        });
        methods.add_method_mut("int", |_, this, (min, max): (i64, i64)| {
            if min >= max {
                lua_error!("max must be greater than min")
            }
            Ok(this.0.gen_range(min..max))
        });
        methods.add_method_mut("int_i", |_, this, (min, max): (i64, i64)| {
            if min > max {
                lua_error!("max must be greater than or equal to min")
            }
            Ok(this.0.gen_range(min..=max))
        });
        methods.add_method_mut("int_skip", |_, this, (min, skip, max): (i64, i64, i64)| -> LuaResult<i64> {
            if min >= max {
                lua_error!("max must be greater than min")
            }
            if skip < min || skip > max {
                Ok(this.0.gen_range(min..max))
            } else {
                let range_size: u64 = max.abs_diff(min);
                if range_size > 1 {
                    let range_select = this.0.gen_range(1..range_size) % range_size;
                    let output = min.wrapping_add_unsigned(range_select);
                    Ok(output)
                } else {
                    Ok(this.0.gen_range(min..max))
                }
            }
        });
        methods.add_method_mut("int_normal", |_, this, (min, max): (i64, i64)| {
            if min >= max {
                lua_error!("max must be greater than min")
            }
            let (a, b) = (this.0.gen_range(min..max), this.0.gen_range(min..max));
            Ok((a + b) / 2)
        });
        methods.add_method_mut("int_normal_i", |_, this, (min, max): (i64, i64)| {
            if min > max {
                lua_error!("max must be greater than or equal to min")
            }
            let (a, b) = (this.0.gen_range(min..=max), this.0.gen_range(min..=max));
            Ok((a + b) / 2)
        });
        methods.add_method_mut("int_bias_low", |_, this, (min, max): (i64, i64)| {
            if min >= max {
                lua_error!("max must be greater than min")
            }
            let (a, b) = (this.0.gen_range(min..max), this.0.gen_range(min..max));
            Ok(cmp::min(a, b))
        });
        methods.add_method_mut("int_bias_high", |_, this, (min, max): (i64, i64)| {
            if min >= max {
                lua_error!("max must be greater than min")
            }
            let (a, b) = (this.0.gen_range(min..max), this.0.gen_range(min..max));
            Ok(cmp::max(a, b))
        });
        methods.add_method_mut("ints_unique_i", |lua, this, (n, min, max): (usize, i64, i64)| {
            if min > max {
                lua_error!("min code length cannot be greater than max")
            }
    
            let range_size = (max - min + 1) as usize;
    
            if n > range_size {
                lua_error!("element count ({}) is greater than number of possible values ({})", n, range_size)
            }
    
            let distr = Uniform::new_inclusive::<i64, i64>(min, max);
            let mut set = HashSet::new();
            for _ in 0..n {
                loop {
                    let candidate = this.0.sample(distr);
                    if set.insert(candidate) {
                        break
                    }
                }
            }
            lua.create_table_from(set.into_iter().enumerate())
        });
        methods.add_method_mut("maybe", |_, this, p: f64| {
            match p {
                p if {p < 0.0 || p.is_nan()} => Ok(false),
                p if {p > 1.0} => Ok(true),
                p => Ok(this.0.gen_bool(p))
            }
        });
        methods.add_method_mut("float", |_, this, (min, max): (f64, f64)| {
            if min >= max {
                lua_error!("max must be greater than min")
            }
            Ok(this.0.gen_range(min..max))
        });
        methods.add_method_mut("normal", |_, this, (min, max): (f64, f64)| {
            if min >= max {
                lua_error!("max must be greater than min")
            }
            Ok((this.0.gen_range(min..max) + this.0.gen_range(min..max)) * 0.5)
        });
        methods.add_method_mut("digits", |_, this, n: Option<usize>| {
            let n = n.unwrap_or(1);
            let distr = Uniform::new_inclusive::<u32, u32>(0, 9);
            let digits: String = distr.sample_iter(&mut this.0).take(n).map(|c| char::from_digit(c, 10).unwrap()).collect();
            Ok(digits)
        });
    }
}

pub struct LuaPerlinSampler(PerlinNoise2D);

impl LuaPerlinSampler {
    pub fn new(octaves: i32, frequency: f64, persistence: f64, lacunarity: f64, seed: i32) -> Self {
        Self(PerlinNoise2D::new(octaves, 1.0, frequency, persistence, lacunarity, (1.0, 1.0), 0.0, seed))
    }

    pub fn sample(&self, x: f64, y: f64) -> f64 {
        self.0.get_noise(x, y)
    }
}

impl LuaUserData for LuaPerlinSampler {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("sample", |_, this, (x, y): (f64, f64)| {
            Ok(this.sample(x, y))
        });
    }
}