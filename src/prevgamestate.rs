pub struct PrevGameState {
    pub capture_piece: u8,
    pub is_capture: bool,
    pub castle_available: [bool; 4],
}
