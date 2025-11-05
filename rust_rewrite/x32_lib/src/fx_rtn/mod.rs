
#[derive(Debug)]
pub enum CommandFormat {
    Int,
    Float,
    String,
    StringList(&'static [&'static str]),
}

#[derive(Debug)]
pub enum CommandValue {
    Int(i32),
    Float(f32),
    String(&'static str),
    None,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CommandFlags: u32 {
        const F_GET = 0x0001;
        const F_SET = 0x0002;
        const F_XET = Self::F_GET.bits() | Self::F_SET.bits();
        const F_NPR = 0x0004;
        const F_FND = 0x0008;
    }
}

#[derive(Debug)]
pub struct X32Command {
    pub command: &'static str,
    pub format: CommandFormat,
    pub flags: CommandFlags,
    pub value: CommandValue,
}

pub mod data;

#[cfg(test)]
mod tests;
