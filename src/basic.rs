use crate::world::{BlockPos, World, BlockDirection};


pub fn create_wire(world: &mut World, block: u16, start: BlockPos, end: BlockPos, repeated: bool) {
    let wire = world.add_block("minecraft:redstone_wire");
    let mut ss = 15;
    let dir = start.direction_to(end);
    let repeater_name = match dir {
        BlockDirection::North => "minecraft:repeater[facing=south]",
        BlockDirection::South => "minecraft:repeater[facing=north]",
        BlockDirection::East => "minecraft:repeater[facing=west]",
        BlockDirection::West => "minecraft:repeater[facing=east]",
    };
    let repeater = world.add_block(repeater_name);
    let mut cur = start;
    loop {
        world.set_block(cur, block);

        if ss > 0 || !repeated {
            world.set_block(cur.offset(0, 1, 0), wire);
            ss = 15;
        } else {
            world.set_block(cur.offset(0, 1, 0), repeater);
        }

        if cur == end {
            break;
        }
        cur = cur.offset_dir(dir, 1);
        ss -= 1;
    }
}
