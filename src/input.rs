use std::mem::size_of;

type KeysBitmask = u128;
const KEY_CODE_COUNT: usize = 8 * size_of::<KeysBitmask>();

pub struct InputState {
    pub keys_pressed_bitmask: KeysBitmask,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys_pressed_bitmask: 0,
        }
    }

    #[inline(always)]
    pub fn is_key_pressed(&mut self, key_code: winit::keyboard::KeyCode) -> bool {
        let key_code_usize = key_code as usize;
        assert!(
            key_code_usize < KEY_CODE_COUNT,
            "key_code: {:?} not supported",
            key_code
        );
        self.keys_pressed_bitmask & (1 << key_code_usize) != 0
    }

    #[inline(always)]
    pub fn set_key_pressed(&mut self, key_code: winit::keyboard::KeyCode, pressed: bool) {
        let key_code_usize = key_code as usize;
        assert!(
            key_code_usize < KEY_CODE_COUNT,
            "key_code: {:?} not supported",
            key_code
        );
        self.keys_pressed_bitmask &= !(1 << key_code_usize);
        self.keys_pressed_bitmask |= (pressed as KeysBitmask) << key_code_usize;
    }
}