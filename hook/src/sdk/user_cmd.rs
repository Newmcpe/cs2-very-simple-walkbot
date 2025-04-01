use static_assertions::const_assert_eq;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

macro_rules! pad {
    ($size:expr) => {
        [MaybeUninit<u8>; $size]
    };
}

#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ECommandButtons {
    IN_ATTACK = 1 << 0,
    IN_JUMP = 1 << 1,
    IN_DUCK = 1 << 2,
    IN_FORWARD = 1 << 3,
    IN_BACK = 1 << 4,
    IN_USE = 1 << 5,
    IN_LEFT = 1 << 7,
    IN_RIGHT = 1 << 8,
    IN_MOVELEFT = 1 << 9,
    IN_MOVERIGHT = 1 << 10,
    IN_SECOND_ATTACK = 1 << 11,
    IN_RELOAD = 1 << 13,
    IN_SPRINT = 1 << 16,
    IN_JOYAUTOSPRINT = 1 << 17,
    IN_SHOWSCORES = 1 << 33,
    IN_ZOOM = 1 << 34,
    IN_LOOKATWEAPON = 1 << 35,
}

#[repr(C)]
#[derive(Debug)]
pub struct CBasePB {
    _vtable: pad!(0x8),
    pub n_has_bits: u32,
    pub n_cached_bits: u64,
}
const_assert_eq!(size_of::<CBasePB>(), 0x18);

impl CBasePB {
    pub fn set_bits(&mut self, n_bits: u64) {
        self.n_cached_bits |= n_bits;
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CMsgQAngle {
    pub base: CBasePB,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}
const_assert_eq!(size_of::<CMsgQAngle>(), 0x28);

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RepeatedPtrField<T> {
    arena: *mut std::ffi::c_void,
    current_size: i32,
    total_size: i32,
    rep: *mut Rep<T>,
    _phantom: PhantomData<T>,
}

#[repr(C)]
pub struct Rep<T> {
    allocated_size: i32,
    elements: [*mut T; 0],
}

#[repr(C)]
#[derive(Debug)]
pub struct CInButtonStatePB {
    pub base: CBasePB,
    pub buttonstate1: u64,
    pub buttonstate2: u64,
    pub buttonstate3: u64,
}
const_assert_eq!(size_of::<CInButtonStatePB>(), 0x30);

#[repr(C)]
#[derive(Debug)]
pub struct CBaseUserCmdPB<'a> {
    pub base: CBasePB,
    pub subtick_moves_field: RepeatedPtrField<CSubtickMoveStep>,
    pub str_move_crc: *mut String,
    pub in_button_state: &'a mut CInButtonStatePB,
    pub view_angles: &'a mut CMsgQAngle,
    pub legacy_command_number: i32,
    pub client_tick: i32,
    pub forward_move: f32,
    pub side_move: f32,
    pub up_move: f32,
    pub impulse: i32,
    pub weapon_select: i32,
    pub random_seed: i32,
    pub moused_x: i32,
    pub moused_y: i32,
    pub consumed_server_angle_changes: u32,
    pub cmd_flags: i32,
    pub pawn_entity_handle: u32,
}
const_assert_eq!(size_of::<CBaseUserCmdPB>(), 0x80);

#[repr(C)]
pub struct CSGOInterpolationInfoPB {
    pub base: CBasePB,
    pub fraction: f32, // 0x18
    pub src_tick: i32, // 0x1C
    pub dst_tick: i32, // 0x20
}
const_assert_eq!(size_of::<CSGOInterpolationInfoPB>(), 0x28);

#[repr(C)]
#[derive(Debug)]
pub struct CSGOInputHistoryEntryPB {
    pub base: CBasePB,
    pub view_angles: *mut CMsgQAngle,                // 0x18
    pub shoot_position: *mut CMsgQAngle,             // 0x20
    pub target_head_position_check: *mut CMsgQAngle, // 0x28
    pub target_abs_position_check: *mut CMsgQAngle,  // 0x30
    pub target_ang_position_check: *mut CMsgQAngle,  // 0x38
    pub cl_interp: *mut CSGOInterpolationInfoPB,     // 0x40
    pub sv_interp0: *mut CSGOInterpolationInfoPB,    // 0x48
    pub sv_interp1: *mut CSGOInterpolationInfoPB,    // 0x50
    pub player_interp: *mut CSGOInterpolationInfoPB, // 0x58
    pub render_tick_count: i32,                      // 0x60
    pub render_tick_fraction: f32,                   // 0x64
    pub player_tick_count: i32,                      // 0x68
    pub player_tick_fraction: f32,                   // 0x6C
    pub frame_number: i32,                           // 0x70
    pub target_ent_index: i32,                       // 0x74
}
const_assert_eq!(size_of::<CSGOInputHistoryEntryPB>(), 0x78);

#[repr(C)]
#[derive(Debug)]
pub struct CSGOUserCmdPB<'a> {
    pub has_bits: u32,
    pub cached_size: u64,
    pub input_history: RepeatedPtrField<CSGOInputHistoryEntryPB>,
    pub base: &'a mut CBaseUserCmdPB<'a>,
    pub attack3_start_history_index: i32,
    pub attack1_start_history_index: i32,
    pub attack2_start_history_index: i32,
}
const_assert_eq!(size_of::<CSGOUserCmdPB>(), 0x40);

#[repr(C)]
#[derive(Debug)]
pub struct CSubtickMoveStep {
    pub base: CBasePB,
    pub n_button: u64,
    pub b_pressed: bool,
    pub fl_when: f32,
    pub fl_analog_forward_delta: f32,
    pub fl_analog_left_delta: f32,
}
const_assert_eq!(size_of::<CSubtickMoveStep>(), 0x30);

#[repr(C)]
#[derive(Debug)]
pub struct CInButtonState {
    _vtable: [MaybeUninit<u8>; 0x8],
    pub n_value: u64,
    pub n_value_changed: u64,
    pub n_value_scroll: u64,
}
const_assert_eq!(size_of::<CInButtonState>(), 0x20);

#[repr(C)]
#[derive(Debug)]
pub struct CUserCmd<'a> {
    _vtable: pad!(0x8),
    _pad: pad!(0x10),
    pub csgo_user_cmd: CSGOUserCmdPB<'a>,
    pub buttons: CInButtonState,
    _pad2: pad!(0x10),
    pub m_bHasBeenPredicted: bool,
    _pad3: pad!(0xF),
}
const_assert_eq!(size_of::<CUserCmd>(), 0x98);

pub enum EButtonStatePBBits {
    BUTTON_STATE_PB_BITS_BUTTONSTATE1 = 0x1,
    BUTTON_STATE_PB_BITS_BUTTONSTATE2 = 0x2,
    BUTTON_STATE_PB_BITS_BUTTONSTATE3 = 0x4,
}
#[repr(C)]
#[derive(Debug)]
pub struct CUserCmdManager<'a> {
    arr_commands: [CUserCmd<'a>; 150],
    _pad: [MaybeUninit<u8>; 0x752],
    n_sequence_number: i32,
    n_previous_sequence_number: i32,
}
impl<'a> CUserCmdManager<'a> {
    pub fn get_user_cmd(&mut self) -> &mut CUserCmd<'a> {
        &mut self.arr_commands[self.n_sequence_number as usize % 150]
    }
}
