#[derive(Debug, Clone, Copy)]
pub enum PlayerState {
    Idle = 0,
    Walking = 1,
    Attacking = 2,
    Dashing = 3,
}
