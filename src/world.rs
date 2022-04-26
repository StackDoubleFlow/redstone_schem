use std::{collections::HashMap, fs::File};
use serde::Serialize;

pub const MC_DATA_VERSION: i32 = 2730;

#[derive(Clone, Copy, Debug)]
pub struct BlockPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl BlockPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }
}

pub struct World {
    sx: usize,
    sy: usize,
    sz: usize,
    data: Vec<u16>,
    palette: HashMap<&'static str, u16>,
}

impl World {
    pub fn new(sx: usize, sy: usize, sz: usize) -> Self {
        let mut palette = HashMap::new();
        palette.insert("minecraft:air", 0);
        Self {
            sx, sy, sz, data: vec![0; sx * sy * sz], palette
        }
    }

    pub fn add_block(&mut self, name: &'static str) -> u16 {
        let new = self.palette.len();
        *self.palette.entry(name).or_insert(new as u16)
    }

    pub fn set_block(&mut self, pos: BlockPos, block: u16) {
        let idx = (self.sx * self.sy * pos.z) + (self.sx * pos.y) + pos.x;
        self.data[idx] = block;
    }

    fn get_block(&self, pos: BlockPos) -> u16 {
        let idx = (self.sx * self.sy * pos.z) + (self.sx * pos.y) + pos.x;
        self.data[idx]
    }

    pub fn save_schematic(&self, file_name: &str) {
        let mut file = File::create(file_name).unwrap();
    
        let mut data = Vec::new();
        for y in 0..self.sy {
            for z in 0..self.sz {
                for x in 0..self.sx {
                    let mut idx = self.get_block(BlockPos::new(x, y, z));
    
                    loop {
                        let mut temp = (idx & 0b1111_1111) as u8;
                        idx >>= 7;
                        if idx != 0 {
                            temp |= 0b1000_0000;
                        }
                        data.push(temp as i8);
                        if idx == 0 {
                            break;
                        }
                    }
                }
            }
        }
    
        let mut encoded_pallete = nbt::Blob::new();
        for (&entry, &i) in &self.palette {
            encoded_pallete.insert(entry, i as i32).unwrap();
        }
    
        // TODO
        let block_entities = Vec::new();
        // for (pos, block_entity) in &clipboard.block_entities {
        //     if let Some(mut blob) = block_entity.to_nbt(false) {
        //         blob.insert("Pos", nbt::Value::IntArray(vec![pos.x, pos.y, pos.z]))?;
        //         block_entities.push(blob);
        //     }
        // }
    
        let metadata = Metadata {
            offset_x: 0,
            offset_y: 0,
            // offset_y: -(self.sy as i32),
            offset_z: 0,
        };
        let schematic = Schematic {
            width: self.sx as i16,
            length: self.sz as i16,
            height: self.sy as i16,
            block_data: data,
            block_entities,
            palette: encoded_pallete,
            metadata,
            version: 2,
            data_version: MC_DATA_VERSION,
        };
        nbt::to_gzip_writer(&mut file, &schematic, Some("Schematic")).unwrap();
    }
}

#[derive(Serialize)]
struct Metadata {
    #[serde(rename = "WEOffsetX")]
    offset_x: i32,
    #[serde(rename = "WEOffsetY")]
    offset_y: i32,
    #[serde(rename = "WEOffsetZ")]
    offset_z: i32,
}

/// Used to serialize schematics in NBT. This cannot be used for deserialization because of
/// [a bug](https://github.com/PistonDevelopers/hematite_nbt/issues/45) in `hematite-nbt`.
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Schematic {
    width: i16,
    length: i16,
    height: i16,
    palette: nbt::Blob,
    metadata: Metadata,
    #[serde(serialize_with = "nbt::i8_array")]
    block_data: Vec<i8>,
    block_entities: Vec<nbt::Blob>,
    version: i32,
    data_version: i32,
}