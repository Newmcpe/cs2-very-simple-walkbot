#![allow(unsafe_op_in_unsafe_fn)]
use imgui::{ConfigFlags, Context, ImColor32, Key};
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::Mutex;
use std::cell::RefCell;
use std::mem::transmute;

use crate::create_move::{CURRENT_POS_INDEX, POSITIONS};
use crate::{cleanup_resources, utils};
use std::sync::atomic::Ordering;
use std::sync::{Once, OnceLock};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct3D11::{ID3D11DeviceContext, ID3D11RenderTargetView};
use windows::Win32::Graphics::Dxgi::{DXGI_PRESENT, IDXGISwapChain};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging;
use windows::Win32::UI::WindowsAndMessaging::{GWLP_WNDPROC, SetWindowLongPtrW, WNDPROC};
use windows::core::{HRESULT, s};

pub(crate) type DXGISwapChainPresentType = unsafe extern "system" fn(
    this: *const IDXGISwapChain,
    sync_interval: u32,
    flags: DXGI_PRESENT,
) -> HRESULT;

pub static ORIGINAL_PRESENT: OnceLock<DXGISwapChainPresentType> = OnceLock::new();

static WND_PROC: OnceCell<WNDPROC> = OnceCell::new();
static DEVICE: OnceCell<&ID3D11DeviceContext> = OnceCell::new();
static TARGET_VIEW: OnceCell<&ID3D11RenderTargetView> = OnceCell::new();
static ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));

static INIT_COMPLETE: Once = Once::new();

thread_local! {
    static IMGUI: RefCell<Option<Context>> = RefCell::new(None);
}

fn wnd_proc_proxy(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if unsafe { utils::imgui::wnd_proc(hwnd, msg, wparam, lparam).0 } != 0 {
        return LRESULT(true.into());
    }
    {
        let enabled = ENABLED.lock();

        if *enabled {
        } else if let Some(wnd_proc) = WND_PROC.get() {
            drop(enabled);

            return unsafe {
                WindowsAndMessaging::CallWindowProcW(*wnd_proc, hwnd, msg, wparam, lparam)
            };
        }
    }

    unsafe { WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam) }
}
pub unsafe extern "system" fn hk_present(
    this: *const IDXGISwapChain,
    sync_interval: u32,
    flags: DXGI_PRESENT,
) -> HRESULT {
    INIT_COMPLETE.call_once(|| {
        let swap_chain = unsafe { this.as_ref() }.unwrap();
        let (device, target_view) = unsafe {
            let device = utils::interop::device(swap_chain).as_ref().unwrap();
            let buf = utils::interop::buf(swap_chain).as_ref().unwrap();
            let target_view = utils::interop::create_render_target(device, buf)
                .as_ref()
                .unwrap();

            (device, target_view)
        };

        let mut imgui = Context::create();
        let io = imgui.io_mut();
        io.config_flags = ConfigFlags::NO_MOUSE_CURSOR_CHANGE;

        let window = utils::interop::desc(this).OutputWindow;

        let wnd_proc: WNDPROC = unsafe {
            let wnd_proc =
                SetWindowLongPtrW(window, GWLP_WNDPROC, wnd_proc_proxy as usize as isize);

            transmute(wnd_proc)
        };

        let device = unsafe {
            let device_ctx = utils::interop::immediate_context(device).as_ref().unwrap();
            utils::imgui::init(window, device, device_ctx);
            device_ctx
        };

        WND_PROC.get_or_init(|| wnd_proc);
        DEVICE.get_or_init(|| device);
        TARGET_VIEW.get_or_init(|| target_view);

        IMGUI.with(|f| {
            *f.borrow_mut() = Some(imgui);
        });
    });

    IMGUI.with_borrow_mut(|f| {
        if let (Some(imgui), Some(device), Some(target_view)) = (f, DEVICE.get(), TARGET_VIEW.get())
        {
            unsafe {
                utils::imgui::frame();
            }

            let ui = imgui.frame();

            {
                let mut enabled = ENABLED.lock();

                if ui.is_key_pressed(Key::Insert) {
                    *enabled = !*enabled;
                }

                if *enabled {
                    ui.window("Walkbot")
                        .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            if ui.button("Unload") {
                                cleanup_resources();
                                SetWindowLongPtrW(
                                    utils::interop::desc(this).OutputWindow,
                                    GWLP_WNDPROC,
                                    WND_PROC.get_unchecked().unwrap() as isize,
                                );
                            }
                        });
                }

                let draw_list = ui.get_background_draw_list();
                let enabled = crate::create_move::ENABLED.load(Ordering::SeqCst);
                if enabled {
                    let current_index = CURRENT_POS_INDEX.load(Ordering::SeqCst);
                    let positions = POSITIONS.get().unwrap().lock().unwrap();
                    let target_pos = positions[current_index].clone();
                    draw_list.add_text(
                        [10.0, 10.0],
                        ImColor32::from_rgb(255, 255, 255),
                        "Going at: ".to_string()
                            + &format!(
                                "{:.2}, {:.2}, {:.2}",
                                target_pos.x, target_pos.y, target_pos.z
                            ),
                    );
                    draw_list.add_text(
                        [10.0, 20.0],
                        ImColor32::from_rgb(255, 255, 255),
                        "Current index: ".to_string() + &format!("{}", current_index),
                    );
                    drop(positions);
                }
            }

            let draw_data = imgui.render();

            utils::interop::render_target(*device, *target_view);
            utils::imgui::render(draw_data);
        }
    });

    ORIGINAL_PRESENT.get().unwrap()(this, sync_interval, flags)
}

pub unsafe fn get_target_address() -> usize {
    let discord_overlay_module = GetModuleHandleA(s!("DiscordHook64.dll"));
    discord_overlay_module.unwrap().0 as usize + 0x1050E0
}
