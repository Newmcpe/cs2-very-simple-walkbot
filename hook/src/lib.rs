#![allow(unsafe_op_in_unsafe_fn)]
mod create_move;
mod render;
mod sdk;
mod utils;

use crate::create_move::{ORIGINAL_CREATEMOVE, get_local_player_pawn};
use crate::render::ORIGINAL_PRESENT;
use minhook::MinHook;
use std::ffi::c_void;
use std::panic::set_hook;
use std::sync::OnceLock;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows::Win32::System::Threading::Sleep;
use windows::core::{BOOL, s};

static CLIENT_DLL: OnceLock<usize> = OnceLock::new();

pub unsafe fn cleanup_resources() {
    unsafe {
        Sleep(3000);
        FreeConsole().unwrap();
        MinHook::disable_all_hooks().unwrap();
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    _hInstDll: HINSTANCE,
    fdwReason: u32,
    _lpvReserved: *mut c_void,
) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            AllocConsole().unwrap();
            set_hook(Box::new(|panic_info| {
                println!("panic: {:?}", panic_info);
                loop {}
            }));
            CLIENT_DLL.get_or_init(|| {
                let module = GetModuleHandleA(s!("client.dll")).unwrap();
                module.0 as usize
            });

            ORIGINAL_PRESENT.get_or_init(|| {
                std::mem::transmute(
                    MinHook::create_hook(
                        *(render::get_target_address() as *mut *mut c_void),
                        render::hk_present as _,
                    )
                    .unwrap(),
                )
            });

            ORIGINAL_CREATEMOVE.get_or_init(|| {
                std::mem::transmute(
                    MinHook::create_hook(
                        create_move::get_createmove_address() as *mut c_void,
                        create_move::hk_createmove as _,
                    )
                    .unwrap(),
                )
            });

            let player_pawn = get_local_player_pawn();
            println!("Player pawn at address: {:#x}", player_pawn as usize);

            MinHook::enable_all_hooks().unwrap();
        }
        DLL_PROCESS_DETACH => {
            cleanup_resources();
            println!("DLL detached");
        }
        _ => {}
    }

    BOOL(1)
}

#[cfg(test)]
mod tests {
    use crate::sdk::user_cmd::CUserCmd;

    #[test]
    fn test_dll_main() {
        unsafe {
            let size = size_of::<CUserCmd>();
            println!("CUserCmd size: {:#x}", size);
        }
    }
}
