use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct PacketFlag: u32 {
        const WALK = 0x1;
        const UNK_2 = 0x2;
        const SPAWN_RELATED = 0x4;
        const EXTENDED = 0x8;
        const FACING_LEFT = 0x10;
        const STANDING = 0x20;
        const FIRE_DAMAGE = 0x40;
        const JUMP = 0x80;
        const GOT_KILLED = 0x100;
        const PUNCH = 0x200;
        const PLACE = 0x400;
        const TILE_CHANGE = 0x800;
        const GOT_PUNCHED = 0x1000;
        const RESPAWN = 0x2000;
        const OBJECT_COLLECT = 0x4000;
        const TRAMPOLINE = 0x8000;
        const DAMAGE = 0x10000;
        const SLIDE = 0x20000;
        const PARASOL = 0x40000;
        const UNK_GRAVITY_RELATED = 0x80000;
        const SWIM = 0x100000;
        const WALL_HANG = 0x200000;
        const POWER_UP_PUNCH_START = 0x400000;
        const POWER_UP_PUNCH_END = 0x800000;
        const UNK_TILE_CHANGE = 0x1000000;
        const HAY_CART_RELATED = 0x2000000;
        const ACID_RELATED_DAMAGE = 0x4000000;
        const UNK_3 = 0x8000000;
        const ACID_DAMAGE = 0x10000000;
    }
}