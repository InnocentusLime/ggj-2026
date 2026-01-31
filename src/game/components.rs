#[derive(Debug, Clone, Copy)]
pub enum PlayerState {
    Idle = 0,
}

#[derive(Debug, Clone, Copy)]
pub enum StabberState {
    Active = 0,
}

#[derive(Debug, Clone, Copy)]
pub struct Hunter;

#[derive(Debug, Clone, Copy)]
pub struct CurrentMask(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Mask(pub u32);
