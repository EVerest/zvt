/// Messages for Feig specific commands
#[repr(u16)]
pub enum CVendFunctions {
    SystemsInfo = 1,
    FactoryReset = 0x0255,
}
