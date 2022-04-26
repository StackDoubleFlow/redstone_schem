use world::{World, BlockPos};

mod world;

// Get position of wire with correct grouping
fn byte_pos(mut pos: BlockPos) -> BlockPos {
    let byte = pos.y / 16;
    let offset = pos.y % 16;
    pos.y = byte * 20 + offset;
    pos
}

fn wire_block(world: &mut World, mut pos: BlockPos, block: u16) {
    let wire = world.add_block("minecraft:redstone_wire");
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
        if height > 2 {
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
    let mut pos = start;
    for i in 0..length {
        pos.x = start.x + i;
        for b in 0..bits {
            pos.y = start.y + b * 2;
            wire_block(world, byte_pos(pos), concrete);
        }
    }
}

fn connect_bits(world: &mut World, x: usize, a: usize, b: usize) {
    let concrete = world.add_block("minecraft:gray_concrete");
    let repeater = world.add_block("minecraft:repeater[facing=north]");
    let target = world.add_block("minecraft:target");

    if a == b {
        let mut pos = byte_pos(BlockPos::new(x, a * 2, 1));
        for _ in 0..6 {
            wire_block(world, pos, concrete);
            pos.z += 1;
        }
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

fn main() {
    let mut world = World::new(10, 90, 10);
    bus(&mut world, BlockPos::new(0, 0, 0), 16, 10);
    bus(&mut world, BlockPos::new(0, 0, 7), 32, 10);
    connect_bits(&mut world, 3, 5, 17);
    connect_bits(&mut world, 5, 7, 7);
    connect_bits(&mut world, 7, 7, 8);
    connect_bits(&mut world, 9, 2, 3);
    world.save_schematic("test.schem");
}
