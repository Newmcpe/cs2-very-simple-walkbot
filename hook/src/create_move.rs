use crate::CLIENT_DLL;
use crate::sdk::user_cmd::{CUserCmd, ECommandButtons};
use crate::sdk::{QAngle, Vector3};
use crate::utils::keytracker::KeyStateTracker;
use std::cell::RefCell;
use std::f32::consts::PI;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_NUMPAD1, VK_NUMPAD2};

type OnCreateMove = extern "fastcall" fn(input: *const c_void, edx: u32, cmd: *mut CUserCmd) -> f64;
pub static ORIGINAL_CREATEMOVE: OnceLock<OnCreateMove> = OnceLock::new();
pub static ENABLED: AtomicBool = AtomicBool::new(false);
pub static POSITIONS: OnceLock<Mutex<Vec<Vector3>>> = OnceLock::new();
pub static CURRENT_POS_INDEX: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static KEY_TRACKER: RefCell<KeyStateTracker> = RefCell::new(KeyStateTracker::new());
}

#[inline(always)]
fn get_positions() -> &'static Mutex<Vec<Vector3>> {
    POSITIONS.get_or_init(|| Mutex::new(Vec::with_capacity(100)))
}

pub fn vector_angles(forward: &Vector3) -> QAngle {
    let mut angles = QAngle {
        pitch: 0.0,
        yaw: 0.0,
        roll: 0.0,
    };

    if forward.x == 0.0 && forward.y == 0.0 {
        angles.yaw = 0.0;
        if forward.z > 0.0 {
            angles.pitch = 270.0;
        } else {
            angles.pitch = 90.0;
        }
    } else {
        let mut yaw = forward.y.atan2(forward.x) * 180.0 / PI;
        if yaw < 0.0 {
            yaw += 360.0;
        }

        let tmp = (forward.x * forward.x + forward.y * forward.y).sqrt();
        let mut pitch = (-forward.z).atan2(tmp) * 180.0 / PI;
        if pitch < 0.0 {
            pitch += 360.0;
        }

        angles.pitch = pitch;
        angles.yaw = yaw;
    }

    angles
}

pub(crate) unsafe fn hk_createmove(input: *const c_void, edx: u32, cmd: &mut CUserCmd) -> f64 {
    let result = ORIGINAL_CREATEMOVE.get().unwrap()(input, edx, cmd);

    let enabled = ENABLED.load(Ordering::SeqCst);

    KEY_TRACKER.with(|key_tracker| {
        let mut key_tracker = key_tracker.borrow_mut();

        if key_tracker.is_key_pressed(VK_NUMPAD2.0 as i32) {
            let pos = get_player_pos();
            let mut positions = get_positions().lock().unwrap();
            positions.push(pos.clone());
            println!(
                "Recorded position: {:.2}, {:.2}, {:.2} - Total: {}",
                pos.x,
                pos.y,
                pos.z,
                positions.len()
            );
        }

        if key_tracker.is_key_pressed(VK_NUMPAD1.0 as i32) {
            if !enabled {
                CURRENT_POS_INDEX.store(0, Ordering::SeqCst);
            }
            ENABLED.store(!enabled, Ordering::SeqCst);
        }
    });

    if enabled {
        let pos = get_player_pos();
        let velocity = get_player_velocity();

        let positions = get_positions().lock().unwrap();
        if positions.is_empty() {
            ENABLED.store(false, Ordering::SeqCst);
            return result;
        }

        let current_index = CURRENT_POS_INDEX.load(Ordering::SeqCst);
        if current_index >= positions.len() {
            ENABLED.store(false, Ordering::SeqCst);
            return result;
        }

        let target_pos = positions[current_index].clone();
        let base_cmd = &mut cmd.csgo_user_cmd.base;

        let delta = target_pos - pos;
        let distance_xz = delta.distance2d();

        if distance_xz > 25.0 {
            let forward = vector_angles(&delta);
            base_cmd.view_angles.yaw = forward.yaw;
            base_cmd.forward_move = 1.0;

            if distance_xz < 20.0 {
                cmd.buttons.n_value |= ECommandButtons::IN_SPRINT as u64;
            }
        } else {
            if velocity.x.abs() < 0.1 && velocity.y.abs() < 0.1 {
                CURRENT_POS_INDEX.fetch_add(1, Ordering::SeqCst);
            } else {
                // Stop movement
                base_cmd.forward_move = 0.0;
            }
        }
    }

    result
}

pub unsafe fn get_createmove_address() -> usize {
    let module = CLIENT_DLL.get().unwrap();
    module + 0x8CE290
}

pub unsafe fn get_local_player_pawn() -> *mut c_void {
    let module = CLIENT_DLL.get().unwrap();
    *((module + 0x188CF70) as *mut *mut c_void)
}

pub unsafe fn get_player_pos() -> Vector3 {
    let player_pawn = get_local_player_pawn();
    let position_ptr = (player_pawn as usize + 0x1324) as *const Vector3;
    (*position_ptr).clone()
}

pub unsafe fn get_player_velocity() -> Vector3 {
    let player_pawn = get_local_player_pawn();
    let velocity_ptr = (player_pawn as usize + 0x3f0) as *const Vector3;
    (*velocity_ptr).clone()
}
