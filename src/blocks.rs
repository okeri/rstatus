/*
  status bar for tiling wms like i3, sway, etc...
  Copyright (C) 2019 Oleg Keri

  This program is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.
  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.
  You should have received a copy of the GNU General Public License
  along with this program.  If not, see <http://www.gnu.org/licenses/>
*/

use super::block::Block;
use serde::Deserialize;
use serde_yaml;
use std::sync::{Arc, Mutex, Once};

macro_rules! all_blocks {
    ($mac:ident) => {
        $mac!(
            battery,
            cpuload,
            custom,
            filesystem,
            memory,
            temperature,
            time,
            volume,
            network,
        );
    };
}

macro_rules! use_blocks {
    (
         $($name: ident,)+
    ) => {
        $(use super::$name;)+
    }
}

macro_rules! mod_blocks {
    (
         $($name: ident,)+
    ) => {
        $(mod $name;)+
    }
}

macro_rules! define_blocks {
     (
         $($name: tt,)+
     ) => {
         #[allow(non_camel_case_types)]
         #[derive(Deserialize)]
         pub enum Blocks {
             $($name($name::Block)),+
         }

         fn to_box(b: Blocks) -> Box<dyn Block> {
            match b {
                $(Blocks::$name(mut v) => {
                    v.set_name(stringify!($name).to_string());
                    Box::new(v)
                }),+
            }
        }
     }
}

all_blocks! {use_blocks}
all_blocks! {define_blocks}

type BlocksCollection = Vec<Box<dyn Block>>;
pub type BlocksWrapper = Arc<Mutex<BlocksCollection>>;

pub fn blocks() -> BlocksWrapper {
    static mut SINGLETON: *const BlocksWrapper = 0 as *const BlocksWrapper;
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let singleton = Arc::new(Mutex::new(init_blocks()));
            SINGLETON = std::mem::transmute(Box::new(singleton));
        });
        (*SINGLETON).clone()
    }
}

fn init_from_config(config_path: &std::path::Path) -> BlocksCollection {
    let data =
        std::fs::read_to_string(config_path.to_str().unwrap()).expect("cannot find config file");
    let mut vals: Vec<Blocks> = serde_yaml::from_str(&data).expect("failed parse config file");
    let mut result: BlocksCollection = vals.drain(..).map(to_box).collect();
    let mut index = 0;
    for block in result.iter_mut() {
        block.set_index(index);
        index += 1;
    }
    result
}

fn init_blocks() -> BlocksCollection {
    let home = std::env::var_os("HOME").expect("error getting home var");
    let cfg_path = std::path::Path::new(&home)
        .join(".config")
        .join("rstatus")
        .join("config.yaml");

    init_from_config(&cfg_path)
}

pub fn display_all() {
    print!("[");
    let mut first = true;
    let blocks = blocks();
    let mut prev_bg: Option<u32> = None;
    for block in blocks.lock().unwrap().iter() {
        if !first {
            print!(",");
        } else {
            first = false
        }
        block.render(prev_bg);
        prev_bg = block.bgcolor();
    }
    println!("],");
}

pub fn update_by_index(index: usize) {
    blocks().lock().unwrap()[index].update();
    display_all();
}
