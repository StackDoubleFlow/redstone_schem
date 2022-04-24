use world::{World, BlockPos};

mod world;

fn wire_block(world: &mut World, mut pos: BlockPos, block: u16) {
    let wire = world.add_block("minecraft:redstone_wire");
    world.set_block(pos, block);
    pos.y += 1;
    world.set_block(pos, wire);
}

fn tower(world: &mut World, start: BlockPos, height: usize) {
    let slab = world.add_block("minecraft:smooth_stone_slab[type=top]");
    for i in 0..=height / 2 {
        let mut pos = start;
        pos.y += i * 2;
        wire_block(world, pos, slab);
    }
    for i in 0..=(height - 1) / 2 {
        let mut pos = start;
        pos.x += 1;
        pos.y += i * 2 + 1;
        wire_block(world, pos, slab);
    }
}

fn main() {
    let mut world = World::new(10, 10, 10);
    tower(&mut world, BlockPos::new(0, 0, 0), 6);
    tower(&mut world, BlockPos::new(2, 0, 0), 7);
    world.save_schematic("test.schem");
}
