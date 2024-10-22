use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub const MC_DATA_VERSION: i32 = 2730;

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key.into(), $value);
            )+
            m
        }
     };
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl BlockPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn direction_to(self, other: BlockPos) -> BlockDirection {
        match (self.x.cmp(&other.x), self.z.cmp(&other.z)) {
            (Ordering::Greater, _) => BlockDirection::West,
            (Ordering::Less, _) => BlockDirection::East,
            (_, Ordering::Greater) => BlockDirection::North,
            (_, Ordering::Less) => BlockDirection::South,
            _ => unreachable!(),
        }
    }

    pub fn offset(self, x: isize, y: isize, z: isize) -> Self {
        Self {
            x: (self.x as isize + x) as usize,
            y: (self.y as isize + y) as usize,
            z: (self.z as isize + z) as usize,
        }
    }

    pub fn offset_dir(self, dir: BlockDirection, amt: isize) -> BlockPos {
        match dir {
            BlockDirection::West => self.offset(-amt, 0, 0),
            BlockDirection::East => self.offset(amt, 0, 0),
            BlockDirection::North => self.offset(0, 0, -amt),
            BlockDirection::South => self.offset(0, 0, amt),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockDirection {
    North,
    South,
    East,
    West,
}

pub struct World {
    sx: usize,
    sy: usize,
    sz: usize,
    data: Vec<u16>,
    palette: HashMap<&'static str, u16>,
    barrels: HashMap<BlockPos, u32>,
}

impl World {
    pub fn new(sx: usize, sy: usize, sz: usize) -> Self {
        let mut palette = HashMap::new();
        palette.insert("minecraft:air", 0);
        Self {
            sx,
            sy,
            sz,
            data: vec![0; sx * sy * sz],
            palette,
            barrels: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, name: &'static str) -> u16 {
        let new = self.palette.len();
        *self.palette.entry(name).or_insert(new as u16)
    }

    pub fn set_block(&mut self, pos: BlockPos, block: u16) {
        if pos.x >= self.sx || pos.y >= self.sy || pos.z >= self.sz {
            panic!("out of bounds set_block to {:?}", pos);
        }
        let idx = (self.sx * self.sy * pos.z) + (self.sx * pos.y) + pos.x;
        self.data[idx] = block;
    }

    pub fn get_block(&self, pos: BlockPos) -> u16 {
        let idx = (self.sx * self.sy * pos.z) + (self.sx * pos.y) + pos.x;
        self.data[idx]
    }

    pub fn set_barrel(&mut self, pos: BlockPos, ss: u32) {
        let barrel = self.add_block("minecraft:barrel");
        self.set_block(pos, barrel);
        self.barrels.insert(pos, ss);
    }

    pub fn save_schematic(&self, file_name: &str, off_x: i32, off_y: i32, off_z: i32) {
        let mut file = File::create(file_name).unwrap();
        let data = self.data(off_x, off_y, off_z);
        file.write_all(&data).unwrap();
    }

    pub fn data(&self, off_x: i32, off_y: i32, off_z: i32) -> Vec<u8> {
        let mut out = Vec::new();

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

        let mut block_entities = Vec::new();
        for (pos, &ss) in &self.barrels {
            let slots = 27;
            let items_needed = match ss {
                0 => 0,
                15 => slots * 64,
                _ => ((32 * slots * ss as u32) as f32 / 7.0 - 1.0).ceil() as u32,
            } as usize;
            let mut blob = nbt::Blob::new();
            blob.insert("Id", nbt::Value::String("minecraft:barrel".to_string()))
                .unwrap();
            blob.insert(
                "Pos",
                nbt::Value::IntArray(vec![pos.x as i32, pos.y as i32, pos.z as i32]),
            )
            .unwrap();

            let mut items = Vec::new();
            for (slot, items_added) in (0..items_needed).step_by(64).enumerate() {
                let count = (items_needed - items_added).min(64);
                items.push(nbt::Value::Compound(map! {
                    "Count" => nbt::Value::Byte(count as i8),
                    "id" => nbt::Value::String("minecraft:redstone".to_owned()),
                    "Slot" => nbt::Value::Byte(slot as i8)
                }));
            }
            blob.insert("Items", nbt::Value::List(items)).unwrap();
            block_entities.push(blob);
        }

        let metadata = Metadata {
            offset_x: off_x as i32,
            offset_y: off_y as i32,
            offset_z: off_z as i32,
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
        nbt::to_gzip_writer(&mut out, &schematic, Some("Schematic")).unwrap();

        out
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
