//! Generation for RISC-V standard compressed instruction-set (RVC) decoders

use crate::basic::create_wire;
use crate::world::{BlockPos, World};

const CROSS_WIRE: &str = "minecraft:redstone_wire[north=side,east=side,west=side,south=side]";

// Get position of wire with correct grouping
fn byte_pos(mut pos: BlockPos) -> BlockPos {
    let byte = pos.y / 16;
    let offset = pos.y % 16;
    pos.y = byte * 20 + offset;
    pos
}

fn wire_block(world: &mut World, mut pos: BlockPos, block: u16) {
    let wire = world.add_block(CROSS_WIRE);
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

fn bus(world: &mut World, start: BlockPos, bits: usize, length: usize, repeated: bool) {
    let concrete = world.add_block("minecraft:gray_concrete");
    for i in 0..bits {
        let start = byte_pos(start.offset(0, i as isize * 2, 0));
        let end = start.offset(length as isize, 0, 0);
        create_wire(world, concrete, start, end, repeated);
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

fn connect_bit_range(
    world: &mut World,
    bit_slot: &mut [usize],
    start: usize,
    end: usize,
    to: usize,
) {
    for (i, bit) in (start..=end).enumerate() {
        let a = bit;
        let b = to + i;
        println!("{} -> {}", a, b);
        let slot = *bit_slot[a..=b].iter().max().unwrap();

        connect_bits(world, slot * 2, a, b);

        bit_slot[a..=b].iter_mut().for_each(|s| *s = slot + 1);
    }
}

fn extend_bit(
    world: &mut World,
    bit_slot: &mut [usize],
    bit: usize,
    start: usize,
    end: usize,
) {
    let slot = *bit_slot[bit..=end].iter().max().unwrap();
    connect_bits(world, slot * 2, bit, end);

    let concrete = world.add_block("minecraft:gray_concrete");
    let repeater = world.add_block("minecraft:repeater[facing=north]");
    let slab = world.add_block("minecraft:smooth_stone_slab[type=top]");
    for b in start + 1..end {
        let pos = byte_pos(BlockPos::new(slot * 2, b * 2, 6));
        world.set_block(pos, concrete);
        world.set_block(pos.offset(0, 1, 0), repeater);
        if b % 16 == 1 {
            // use slab for bit below
            wire_block(world, pos.offset(0, 0, -1), slab);
        } else {
            wire_block(world, pos.offset(0, 0, -1), concrete);
        }

        // change slab to block 
        if b % 16 == 0 {
            world.set_block(pos.offset(0, 1, -2), concrete);
        }
    }

    bit_slot[bit..=end].iter_mut().for_each(|s| *s = slot + 1);
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

fn gen_ins<F>(name: &str, f: F)
where
    F: FnOnce(&mut World, &mut [usize; 32])
{
    let mut world = World::new(48, 76, 10);
    let mut bit_slot = [0; 32];

    f(&mut world, &mut bit_slot);

    let length = *bit_slot.iter().max().unwrap() * 2;

    let concrete = world.add_block("minecraft:gray_concrete");
    let wall_torch = world.add_block("minecraft:redstone_wall_torch[facing=south]");
    let torch = world.add_block("minecraft:redstone_torch");
    for i in 0..32 {
        let pos = byte_pos(BlockPos::new(length, i * 2 + 1, 7));
        world.set_block(pos, concrete);
        world.set_block(pos.offset(0, 0, 1), wall_torch);
    }
    for i in 0..4 {
        let y = i * 16;
        wire_block(&mut world, byte_pos(BlockPos::new(length, y, 6)), concrete);
        tower(&mut world, BlockPos::new(length, y + 1, 5), 13);

        // layer repeater
        if i < 3 {
            let pos = byte_pos(BlockPos::new(length, y, 7)).offset(0, 16, 0);
            world.set_block(pos, torch);
            wire_block(&mut world, pos.offset(0, 1, 0), concrete);
            world.set_block(pos.offset(0, 2, -1), concrete);
            world.set_block(pos.offset(0, 3, -1), torch);
        }
    }
    

    bus(&mut world, BlockPos::new(0, 0, 0), 16, length + 1, false);
    bus(&mut world, BlockPos::new(0, 0, 7), 32, length - 1, true);
    bus(&mut world, BlockPos::new(0, 0, 9), 32, length + 1, false);

    // let repeater = world.add_block("minecraft:repeater[facing=west]");
    // for i in 0..32 {
    //     world.set_block(byte_pos(BlockPos::new(length, i * 2 + 1, 0)), repeater);
    // }

    world.save_schematic(&format!("rvc/rvc_{}.schem", name));
}

pub fn gen_rvc() {
    gen_ins("lwsp", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 3, 26);
        connect_bit_range(world, bit_slot, 4, 6, 22);
        connect_bit_range(world, bit_slot, 12, 12, 25);
        connect_bit_range(world, bit_slot, 7, 11, 7);

        constant_range(world, 0, 0b0000011);
        constant_range(world, 12, 0b010); // funct3
        constant_range(world, 15, 0b00010); // x2/sp
    });
    gen_ins("swsp", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 6, 20);
        connect_bit_range(world, bit_slot, 7, 8, 26);
        connect_bit_range(world, bit_slot, 9, 11, 9);
        connect_bit_range(world, bit_slot, 12, 12, 25);

        constant_range(world, 0, 0b0100011);
        constant_range(world, 12, 0b010); // funct3
        constant_range(world, 15, 0b00010); // x2/sp
    });
    gen_ins("lw", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 4, 7);
        connect_bit_range(world, bit_slot, 5, 5, 26);
        connect_bit_range(world, bit_slot, 6, 6, 22);
        connect_bit_range(world, bit_slot, 7, 9, 15);
        connect_bit_range(world, bit_slot, 10, 12, 23);

        constant_range(world, 0, 0b0000011);
        constant_range(world, 12, 0b010); // funct3
    });
    gen_ins("sw", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 4, 20);
        connect_bit_range(world, bit_slot, 5, 5, 26);
        connect_bit_range(world, bit_slot, 6, 6, 9);
        connect_bit_range(world, bit_slot, 7, 9, 15);
        connect_bit_range(world, bit_slot, 10, 11, 10);
        connect_bit_range(world, bit_slot, 12, 12, 25);

        constant_range(world, 0, 0b0100011);
        constant_range(world, 12, 0b010); // funct3
    });
    gen_ins("j", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 2, 25);
        connect_bit_range(world, bit_slot, 3, 5, 21);
        connect_bit_range(world, bit_slot, 6, 6, 27);
        connect_bit_range(world, bit_slot, 7, 7, 26);
        connect_bit_range(world, bit_slot, 8, 8, 30);
        connect_bit_range(world, bit_slot, 9, 10, 28);
        connect_bit_range(world, bit_slot, 11, 11, 24);
        extend_bit(world, bit_slot, 12, 12, 20);
        connect_bit_range(world, bit_slot, 12, 12, 31);

        constant_range(world, 0, 0b1101111);
    });
    gen_ins("jal", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 2, 25);
        connect_bit_range(world, bit_slot, 3, 5, 21);
        connect_bit_range(world, bit_slot, 6, 6, 27);
        connect_bit_range(world, bit_slot, 7, 7, 26);
        connect_bit_range(world, bit_slot, 8, 8, 30);
        connect_bit_range(world, bit_slot, 9, 10, 28);
        connect_bit_range(world, bit_slot, 11, 11, 24);
        extend_bit(world, bit_slot, 12, 12, 20);
        connect_bit_range(world, bit_slot, 12, 12, 31);

        constant_range(world, 0, 0b1101111);
        constant_range(world, 7, 0b00001); // x1/lr
    });
    gen_ins("jr", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 7, 11, 15);

        constant_range(world, 0, 0b1100111);
    });
    gen_ins("jalr", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 7, 11, 15);

        constant_range(world, 0, 0b1100111);
        constant_range(world, 7, 0b00001); // x1/lr
    });
    gen_ins("beqz", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 2, 25);
        connect_bit_range(world, bit_slot, 3, 4, 8);
        connect_bit_range(world, bit_slot, 5, 6, 26);
        connect_bit_range(world, bit_slot, 7, 9, 15);
        connect_bit_range(world, bit_slot, 10, 11, 10);
        connect_bit_range(world, bit_slot, 12, 12, 28);

        constant_range(world, 0, 0b1100011);
    });

    gen_ins("bnez", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 2, 25);
        connect_bit_range(world, bit_slot, 3, 4, 8);
        connect_bit_range(world, bit_slot, 5, 6, 26);
        connect_bit_range(world, bit_slot, 7, 9, 15);
        connect_bit_range(world, bit_slot, 10, 11, 10);
        connect_bit_range(world, bit_slot, 12, 12, 28);

        constant_range(world, 0, 0b1100011);
        constant_range(world, 12, 0b001); // funct3
    });
    gen_ins("li", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 6, 20);
        connect_bit_range(world, bit_slot, 7, 11, 7);
        connect_bit_range(world, bit_slot, 12, 12, 25);

        constant_range(world, 0, 0b0010011);
    });
    gen_ins("lui", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 6, 12);
        connect_bit_range(world, bit_slot, 7, 11, 7);
        extend_bit(world, bit_slot, 12, 17, 31);

        constant_range(world, 0, 0b0110111);
    });
    gen_ins("addi", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 6, 20);
        connect_bit_range(world, bit_slot, 7, 11, 7);
        connect_bit_range(world, bit_slot, 7, 11, 15);
        extend_bit(world, bit_slot, 12, 25, 31);

        constant_range(world, 0, 0b0010011);
    });
    gen_ins("addi16sp", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 2, 25);
        connect_bit_range(world, bit_slot, 3, 4, 27);
        connect_bit_range(world, bit_slot, 5, 5, 26);
        connect_bit_range(world, bit_slot, 6, 6, 24);
        connect_bit_range(world, bit_slot, 7, 11, 7);
        connect_bit_range(world, bit_slot, 7, 11, 15);
        extend_bit(world, bit_slot, 12, 25, 31);

        constant_range(world, 0, 0b0010011);
    });
    gen_ins("addi4spn", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 4, 7);
        connect_bit_range(world, bit_slot, 5, 5, 23);
        connect_bit_range(world, bit_slot, 6, 6, 22);
        connect_bit_range(world, bit_slot, 7, 10, 26);
        connect_bit_range(world, bit_slot, 11, 12, 24);

        constant_range(world, 0, 0b0010011);
    });
    gen_ins("slli", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 6, 20);
        connect_bit_range(world, bit_slot, 7, 11, 7);
        connect_bit_range(world, bit_slot, 7, 11, 15);

        constant_range(world, 0, 0b0010011);
        constant_range(world, 12, 0b001); // funct3
    });
    gen_ins("srli_srai", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 6, 20);
        connect_bit_range(world, bit_slot, 7, 9, 7);
        connect_bit_range(world, bit_slot, 7, 9, 15);

        constant_range(world, 0, 0b0010011);
        constant_range(world, 12, 0b101); // funct3
    });
    gen_ins("andi", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 4, 20);
        connect_bit_range(world, bit_slot, 7, 9, 7);
        connect_bit_range(world, bit_slot, 7, 9, 15);
        extend_bit(world, bit_slot, 12, 25, 31);

        constant_range(world, 0, 0b0010011);
        constant_range(world, 12, 0b101); // funct3
    });
    gen_ins("sub_xor_or_and", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 4, 20);
        connect_bit_range(world, bit_slot, 7, 9, 7);
        connect_bit_range(world, bit_slot, 7, 9, 15);

        constant_range(world, 0, 0b0110011);
    });
    gen_ins("add", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 6, 20);
        connect_bit_range(world, bit_slot, 7, 11, 7);
        connect_bit_range(world, bit_slot, 7, 11, 15);

        constant_range(world, 0, 0b0110011);
    });
    gen_ins("mv", |world, bit_slot| {
        connect_bit_range(world, bit_slot, 2, 6, 20);
        connect_bit_range(world, bit_slot, 7, 11, 7);

        constant_range(world, 0, 0b0110011);
    });
}
