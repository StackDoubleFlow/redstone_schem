use crate::basic::create_wire;
use crate::world::{World, BlockPos};

// Get position of wire with correct grouping
fn byte_pos(mut pos: BlockPos) -> BlockPos {
    let byte = pos.y / 16;
    let offset = pos.y % 16;
    pos.y = byte * 20 + offset;
    pos
}

fn wire_block(world: &mut World, mut pos: BlockPos, block: u16) {
    let wire = world.add_block("minecraft:redstone_wire[north=side,east=side,west=side,south=side]");
    world.set_block(pos, block);
    pos.y += 1;
    world.set_block(pos, wire);
}

fn tower(world: &mut World, start: BlockPos, height: usize) {
    if height == 0 {
        return;
    }
    if start.y % 16 + height >= 15 {
        let mut pos = start;

        let start_height = 14 - start.y % 16;
        tower(world, start, start_height);
        
        let byte = start.y / 16;
        if height - start_height > 2 {
            pos.y = (byte + 1) * 16;
            let height = height - start_height - 2;
            tower(world, pos, height);
        }

        let concrete = world.add_block("minecraft:gray_concrete");
        let up_torch = world.add_block("minecraft:redstone_torch[lit=true]");
        let up_torch_off = world.add_block("minecraft:redstone_torch[lit=false]");
        pos.z -= 1;
        pos.y = byte * 20 + 15;
        world.set_block(pos, concrete);
        pos.y += 1;
        world.set_block(pos, up_torch);
        pos.y += 1;
        wire_block(world, pos, concrete);
        pos.z += 1;
        pos.y += 1;
        wire_block(world, pos, concrete);
        pos.y += 1;
        world.set_block(pos, up_torch_off);
        pos.y += 1;
        world.set_block(pos, concrete);
        return;
    }
    let start = byte_pos(start);

    let slab = world.add_block("minecraft:smooth_stone_slab[type=top]");
    for i in 0..=height / 2 {
        let mut pos = start;
        pos.y += i * 2;
        wire_block(world, pos, slab);
    }
    for i in 0..=(height - 1) / 2 {
        let mut pos = start;
        pos.z += 1;
        pos.y += i * 2 + 1;
        wire_block(world, pos, slab);
    }
}

fn bus(world: &mut World, start: BlockPos, bits: usize, length: usize) {
    let concrete = world.add_block("minecraft:gray_concrete");
    for i in 0..bits {
        let start = byte_pos(start.offset(0, i as isize * 2, 0));
        let end = start.offset(length as isize, 0, 0);
        create_wire(world, concrete, start, end, true);
    }
}

fn connect_bits(world: &mut World, x: usize, a: usize, b: usize) {
    let concrete = world.add_block("minecraft:gray_concrete");
    let repeater = world.add_block("minecraft:repeater[facing=north]");
    let target = world.add_block("minecraft:target");

    if a == b {
        let pos = byte_pos(BlockPos::new(x, a * 2, 1));
        for i in 0..6 {
            wire_block(world, pos.offset(0, 0, i), concrete);
        }
        world.set_block(pos.offset(0, 1, 0), repeater);
        world.set_block(pos.offset(0, 1, 5), repeater);
        return;
    }

    let mut pos = byte_pos(BlockPos::new(x, a * 2, 2));
    if a % 8 == 7 {
        // If it's the last bit, repeater directly into the torch
        pos.z -= 1;
        world.set_block(pos, concrete);
        pos.y += 1;
        world.set_block(pos, repeater);
    } else {
        world.set_block(pos, concrete);
        pos.y += 1;
        world.set_block(pos, repeater);
        pos.z -= 1;
        world.set_block(pos, target);
    }

    let mut pos = byte_pos(BlockPos::new(x, b * 2, 5));
    if b % 8 == 0 {
        pos.z -= 2;
        for _ in 0..4 {
            wire_block(world, pos, concrete);
            pos.z += 1;
        }
        pos.z -= 1;
        pos.y += 1;
        world.set_block(pos, repeater);
    } else {
        wire_block(world, pos, concrete);
        pos.z += 1;
        world.set_block(pos, concrete);
        pos.y += 1;
        world.set_block(pos, repeater);
    }
    tower(world, BlockPos::new(x, a * 2, 3), (b - a) * 2 - 1);
}

fn connect_bit_range(world: &mut World, bit_slot: &mut [usize], start: usize, end: usize, to: usize) {
    for (i, bit) in (start..=end).enumerate() {
        let a = bit;
        let b = to + i;
        println!("{} -> {}", a, b);
        let slot = *bit_slot[a..=b].iter().max().unwrap();

        connect_bits(world, slot * 2, a, b);

        bit_slot[a..=b].iter_mut().for_each(|s| *s = slot + 1);
    }
}

fn constant_range(world: &mut World, start: usize, constant: u32) {
    let redstone_block = world.add_block("minecraft:redstone_block");
    for i in 0..32 {
        let b = constant & (1 << i);
        if b > 0 {
            println!("#1 -> {}", start + i);
            let pos = BlockPos::new(0, (start + i) * 2 + 1, 6);
            world.set_block(byte_pos(pos), redstone_block);
        }
    }
}

pub fn gen_rvc() {
    let mut world = World::new(32, 76, 10);
    let mut bit_slot = [0; 32];

    connect_bit_range(&mut world, &mut bit_slot, 2, 3, 26);
    connect_bit_range(&mut world, &mut bit_slot, 4, 6, 22);
    connect_bit_range(&mut world, &mut bit_slot, 12, 12, 25);
    connect_bit_range(&mut world, &mut bit_slot, 7, 11, 7);

    constant_range(&mut world, 0, 0b0000011);
    constant_range(&mut world, 12, 0b010); // funct3
    constant_range(&mut world, 15, 0b00010); // x2/sp

    let length = *bit_slot.iter().max().unwrap() * 2;
    bus(&mut world, BlockPos::new(0, 0, 0), 16, length);
    bus(&mut world, BlockPos::new(0, 0, 7), 32, length);

    world.save_schematic("test.schem");
}
