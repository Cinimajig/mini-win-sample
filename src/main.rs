#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![no_std]
#![no_main]

// Fixes linkage issues with the CRT.
#[cfg(target_feature = "crt-static")]
#[link(name = "libcmt")]
extern "C" {}
#[cfg(not(target_feature = "crt-static"))]
#[link(name = "msvcrt")]
extern "C" {}

use core::{mem, ptr};
use windows_sys::{
    core::*,
    Win32::{Foundation::*, UI::WindowsAndMessaging::*},
};

// Id's of menuitems.
const IDM_FILE: usize = 100;
const IDM_ABOUT: usize = 101;
const IDM_EXIT: usize = 110;

// About message, converted to a UTF-16 literal.
const ABOUT_MSG: PCWSTR = w!(r#"Version 1.0 

Created for demonstration purposes.


Copyright ©️ LOLOLOL Corp."#);

/// The main function only exists for debug builds. We call WinMain here to have intellisense while coding.
/// Normally WinMain gets it's parameters from Windows on startup.
#[cfg(debug_assertions)]
#[no_mangle]
unsafe extern "system" fn wmain(_argc: i32, argv: PCWSTR) -> i32 {
    use windows_sys::Win32::System::LibraryLoader::*;

    WinMain(GetModuleHandleW(ptr::null()), 0, argv, SW_SHOWDEFAULT)
}

/// The "real" main function of a Windows GUI application. It get's its paramenters from Windows on startup.
///
/// Note: `prev_instance` only exists for backwards compatibility reasons. On 32-bit an up it will always be null-pointer
/// So it should never be used.
#[no_mangle]
unsafe extern "system" fn WinMain(
    instance: HINSTANCE,
    _prev_instance: HINSTANCE,
    _cmdline: PCWSTR,
    show: u32,
) -> i32 {
    use windows_sys::Win32::Graphics::Gdi::*;

    // All Windows have a Window Class. A Window Class is stuff, that is what's common to that type of Window.
    // Even if we only have one Window, we still need to register it.
    let wc = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as _,     // Size of itself.
        style: CS_VREDRAW | CS_HREDRAW,                 // The Window should redraw when the size changes.
        lpfnWndProc: Some(wnd_proc),                    // Our Window Procedure.
        hInstance: instance,                            // Application instance.
        hIcon: LoadIconW(0, IDI_APPLICATION),           // Application Icon (IDI_APPLICATION == Default icon).
        hCursor: LoadCursorW(0, IDC_ARROW),             // What cursor to use (IDC_ARROW == The Arrow).
        hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,    // Background color of the Window. The builtin colors should be added with 1.
        lpszClassName: w!("MyWindowClass"),             // Class name. This should be used with CreateWindow.
        ..mem::zeroed()                                 // Fill the rest with zeroes.
    };

    // If the class registration fails, we return the Error.
    if RegisterClassExW(&wc) == 0 {
        return GetLastError() as _;
    }

    // Creates our window, with the class we just registered.
    let window = CreateWindowExW(
        0,                          // Extended style flags. We don't use any in this example.
        wc.lpszClassName,           // The class name from earlier.
        w!("Hello Windows sample"), // Window Title.
        WS_OVERLAPPEDWINDOW,        // What kind of style our window is (Caption bar, min-max buttons, thick frame and so forth).
        CW_USEDEFAULT,              // Let Windows place the window, wherever it likes.
        CW_USEDEFAULT,              // Let Windows place the window, wherever it likes.
        800,                        // Width of the window.
        600,                        // Height of the window.
        0,                          // Parent window. We don't have any, as this is a toplevel window.
        create_menu(),              // A Handle to our menu.
        instance,                   // Application instance.
        ptr::null(),                // Pointer to any extra data, we wan't to pass to the Window Procedure. We don't have any.
    );

    // If the window creation failed, we return the Error.
    if window == 0 {
        return GetLastError() as _;
    }

    // Tells the application, how it should we showed. If the user had a shortcut that starts it Maximized, we should do that (if possible).
    // Default Windows will pass the value of SW_SHOWDEFAULT, which is 10.
    ShowWindow(window, show);

    // The Message loop of our process. We keep calling `GetMessage`, until it returns a zero.
    let mut msg: MSG = mem::zeroed();
    while GetMessageW(&mut msg, 0, 0, 0) != 0 {
        // If the message we got, was a quit-message, we break out of the loop.
        if msg.message == WM_QUIT {
            break;
        }

        // Here we translates the message into a command, that our Window Procedure can use, and the pass it to it.
        // Continue in the `wnd_proc` function from here.
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    // Returns the WPARAM of the last message (WM_QUIT).
    msg.wParam as _
}

/// A programmatic way of creating a menu to our window. The function returns a Handle, that we can use.
/// If the Handle is passed to `CreateWindow`, it will automatically be cleaned-up by Windows.
unsafe fn create_menu() -> HMENU {
    let menu = CreateMenu();        // Our "top-level" menu.
    let file_menu = CreateMenu();   // Our drop-down menu.

    // Inserts the item "File" into our main menu. This is the drop-down item.
    InsertMenuW(menu, IDM_FILE as _, MF_POPUP, file_menu as _, w!("&File"));

    // Inserts the item "Exit" into the drop-down menu.
    InsertMenuW(
        file_menu,     // Menu to insert into.
        IDM_EXIT as _, // Id or position of the item.
        MF_STRING,     // What type of menuitem.
        IDM_EXIT as _, // Id of the menuitem.
        w!("E&xit"),   // The text content. The "&" decides the Alt-hotkey we can use.
    );

    // Inserts the item "About" into the main menu.
    InsertMenuW(
        menu,
        IDM_ABOUT as _,
        MF_STRING,
        IDM_ABOUT as _,
        w!("&About"),
    );

    // Returns the top-level menu.
    menu
}

/// The Window Procedure. This is the heart of an Windows GUI Application. This function gets called by our the Message loop above.
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    use windows_sys::Win32::Graphics::Gdi::*;

    // If everything goes well, the function should return zero.
    let mut result = 0;

    // We get a Message, we can either handle or not. Everything we don't handle, should be passed to DefWindowProc.
    match msg {
        // WM_COMMAND is when our menuitems are used. We decide here, what to do with them.
        WM_COMMAND => {
            // What item was pressed is contained in the wparam variable. Normally we would check the high bits to determen if it's an 
            // accelerator or control. Since we only have one type in our app, we don't really care.
            match wparam {
                // The About button, shows info about our application. We use a MessageBox instead of a DialogBox, because it's easier.
                IDM_ABOUT => {
                    MessageBoxW(
                        hwnd,
                        ABOUT_MSG,
                        w!("About Hello Windows Sample"),
                        MB_ICONINFORMATION,
                    );
                }
                // When a user presses the Exit button, We call DestroyWindow on the window itself.
                IDM_EXIT => {
                    DestroyWindow(hwnd);
                }
                // If we for some reason get a WM_COMMAND on an item we dont have, we just pass it to DefWindowProc.
                _ => result = DefWindowProcW(hwnd, msg, wparam, lparam),
            };
        }
        // The painting/drawing of the content in window. This happends everytime we told it to do it in the Window Class.
        WM_PAINT => {
            // Creates a RECT and PAINTSTRUCT filled with zeroes.
            let mut rect: RECT = mem::zeroed();
            let mut ps: PAINTSTRUCT = mem::zeroed();

            // Finds out how big the window is, and stores it the variable "rect". We don't know the size beforehand, since it
            // could have been resized.
            GetClientRect(hwnd, &mut rect);

            // Tells the window, it's time to draw. We then recieve a Device Context, we can use it drawing/painting related functions.
            let dc = BeginPaint(hwnd, &mut ps);

            // Removes the white outline of text.
            SetBkMode(dc, TRANSPARENT);

            // Draws som text in the center of the window.
            DrawTextW(
                dc,                                     // Device Context to use.
                w!("Hello, Windows!") as _,             // Our Text.
                -1,                                     // The length of the text. -1 means until a null is encountered.
                &mut rect,                              // A mutable reference to our window size.
                DT_SINGLELINE | DT_VCENTER | DT_CENTER, // Attributes of the text.
            );

            // Tells Windows, we are done painting.
            EndPaint(hwnd, &ps);
        }
        // WM_DESTROY is when our window should be closing. The most important thing is call PostQuitMessage with the value zero.
        // Otherwise, this where most the manual cleanup should happend.
        // This will also make the next message of the Message loop WM_QUIT.
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        // Everything we don't handle, we send to the DefWindowProc function. This will give us default behavior for stuff we don't care about.
        _ => result = DefWindowProcW(hwnd, msg, wparam, lparam),
    }

    // Returns the result.
    result
}

/// Panic handler for our no_std binary. It's a function, that never returns.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
