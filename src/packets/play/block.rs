#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u16)]
pub enum Block {
    Air = 0,
    Bedrock = 33,
    Dirt = 10,
    Grass = 9,
    Water = 49,
    Lava = 50,
    WhiteWool = 1383,
    OrangeWool = 1384,
    MagentaWool = 1385,
    LightBlueWool = 1386,
    YellowWool = 1387,
    LimeWool = 1388,
    PinkWool = 1389,
    GrayWool = 1390,
    LightGrayWool = 1391,
    CyanWool = 1392,
    PurpleWool = 1393,
    BlueWool = 1394,
    BrownWool = 1395,
    GreenWool = 1396,
    RedWool = 1397,
    BlackWool = 1398,
    WhiteStainedGlass = 4081,
    OrangeStainedGlass = 4082,
    MagentaStainedGlass = 4083,
    LightBlueStainedGlass = 4084,
    YellowStainedGlass = 4085,
    LimeStainedGlass = 4086,
    PinkStainedGlass = 4087,
    GrayStainedGlass = 4088,
    LightGrayStainedGlass = 4089,
    CyanStainedGlass = 4090,
    PurpleStainedGlass = 4091,
    BlueStainedGlass = 4092,
    BrownStainedGlass = 4093,
    GreenStainedGlass = 4094,
    RedStainedGlass = 4095,
    BlackStainedGlass = 4096,
    SlimeBlock = 6999,
    WhiteConcrete = 8902,
    OrangeConcrete = 8903,
    MagentaConcrete = 8904,
    LightBlueConcrete = 8905,
    YellowConcrete = 8906,
    LimeConcrete = 8907,
    PinkConcrete = 8908,
    GrayConcrete = 8909,
    LightGrayConcrete = 8910,
    CyanConcrete = 8911,
    PurpleConcrete = 8912,
    BlueConcrete = 8913,
    BrownConcrete = 8914,
    GreenConcrete = 8915,
    RedConcrete = 8916,
    BlackConcrete = 8917,
    WhiteConcretePowder = 8918,
    OrangeConcretePowder = 8919,
    MagentaConcretePowder = 8920,
    LightBlueConcretePowder = 8921,
    YellowConcretePowder = 8922,
    LimeConcretePowder = 8923,
    PinkConcretePowder = 8924,
    GrayConcretePowder = 8925,
    LightGrayConcretePowder = 8926,
    CyanConcretePowder = 8927,
    PurpleConcretePowder = 8928,
    BlueConcretePowder = 8929,
    BrownConcretePowder = 8930,
    GreenConcretePowder = 8931,
    RedConcretePowder = 8932,
    BlackConcretePowder = 8933,
    HoneyBlock = 11335,
}

impl From<u16> for Block {
    fn from(n: u16) -> Block {
        unsafe { std::mem::transmute(n) }
    }
}
