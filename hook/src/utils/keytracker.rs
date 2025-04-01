use std::collections::HashMap;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

#[derive(Clone)]
pub struct KeyStateTracker {
    previous_states: HashMap<i32, bool>,
}

impl KeyStateTracker {
    pub(crate) fn new() -> Self {
        Self {
            previous_states: HashMap::new(),
        }
    }

    // Returns true only on the initial press (key down event)
    pub(crate) unsafe fn is_key_pressed(&mut self, vk_code: i32) -> bool {
        let current_state = (GetAsyncKeyState(vk_code) & 0x8000u16 as i16) != 0;
        let previous_state = *self.previous_states.get(&vk_code).unwrap_or(&false);

        self.previous_states.insert(vk_code, current_state);

        // Return true only when transitioning from not pressed to pressed
        current_state && !previous_state
    }

    // Returns true throughout the entire time the key is held down
    unsafe fn is_key_down(&mut self, vk_code: i32) -> bool {
        (GetAsyncKeyState(vk_code) & 0x8000u16 as i16) != 0
    }

    // Alternative implementation to check if released (going from pressed to not pressed)
    unsafe fn is_key_released(&mut self, vk_code: i32) -> bool {
        let current_state = (GetAsyncKeyState(vk_code) & 0x8000u16 as i16) != 0;
        let previous_state = *self.previous_states.get(&vk_code).unwrap_or(&false);

        self.previous_states.insert(vk_code, current_state);

        // Return true only when transitioning from pressed to not pressed
        !current_state && previous_state
    }
}
