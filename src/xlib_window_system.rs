#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
extern crate libc;

use layout::Rect;
use std::ptr::null_mut;
use std::mem::{uninitialized, transmute};
use std::str::raw::c_str_to_static_slice;
use self::libc::{c_void, c_int, c_char};
use self::libc::funcs::c95::stdlib::malloc;
use xlib::{ Display,
            Window,
            XWindowChanges,
            XOpenDisplay,
            XDefaultRootWindow,
            XSelectInput,
            XDisplayWidth,
            XDisplayHeight,
            XNextEvent,
            XErrorEvent,
            XSetErrorHandler,
            XMapWindow,
            XConfigureWindow,
            XReparentWindow,
            XMoveWindow,
            XResizeWindow,
            XMoveResizeWindow,
            XSetWindowBorderWidth,
            XSetWindowBorder,
            XFetchName,
            XCreateSimpleWindow,
            XMapRequestEvent,
            XConfigureRequestEvent,
            XDestroyWindowEvent,
            XEnterWindowEvent,
            XKeyPressedEvent,
            XLeaveWindowEvent,
            XRaiseWindow
          };

extern fn error_handler(display: *mut Display, event: *mut XErrorEvent) -> c_int {
  // TODO: proper error handling
  // HACK: fixes LeaveNotify on invalid windows
  return 0;
}

const KeyPress               : i32 = 2;
const KeyRelease             : i32 = 3;
const ButtonPress            : i32 = 4;
const ButtonRelease          : i32 = 5;
const MotionNotify           : i32 = 6;
const EnterNotify            : i32 = 7;
const LeaveNotify            : i32 = 8;
const FocusIn                : i32 = 9;
const FocusOut               : i32 = 10;
const KeymapNotify           : i32 = 11;
const Expose                 : i32 = 12;
const GraphicsExpose         : i32 = 13;
const NoExpose               : i32 = 14;
const VisibilityNotify       : i32 = 15;
const CreateNotify           : i32 = 16;
const DestroyNotify          : i32 = 17;
const UnmapNotify            : i32 = 18;
const MapNotify              : i32 = 19;
const MapRequest             : i32 = 20;
const ReparentNotify         : i32 = 21;
const ConfigureNotify        : i32 = 22;
const ConfigureRequest       : i32 = 23;
const GravityNotify          : i32 = 24;
const ResizeRequest          : i32 = 25;
const CirculateNotify        : i32 = 26;
const CirculateRequest       : i32 = 27;
const PropertyNotify         : i32 = 28;
const SelectionClear         : i32 = 29;
const SelectionRequest       : i32 = 30;
const SelectionNotify        : i32 = 31;
const ColormapNotify         : i32 = 32;
const ClientMessage          : i32 = 33;
const MappingNotify          : i32 = 34;

pub struct XlibWindowSystem {
  display:        *mut Display,
  root:           Window,
  event:          *mut c_void
}

pub enum XlibEvent {
  XMapRequest(Window),
  XUnmapNotify(Window),
  XConfigurationRequest(Window, WindowChanges, u64),
  XDestroyNotify(Window),
  XEnterNotify(Window, i32),
  XLeaveNotify(Window, i32),
  XKeyPress(Window, u32, u32),
  Unknown
}


pub struct WindowChanges {
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
  pub border_width: u32,
  pub sibling: Window,
  pub stack_mode: u32,
}

impl XlibWindowSystem {
  pub fn new() -> Option<XlibWindowSystem> {
    unsafe {
      let display = XOpenDisplay(null_mut());
      if display.is_null() {
        return None;
      }

      let root = XDefaultRootWindow(display);
      XSelectInput(display, root, 0x100031);

      XSetErrorHandler(error_handler as *mut u8);

      Some(XlibWindowSystem{
        display: display,
        root: root,
        event: malloc(256)
      })
    }
  }

  pub fn new_vroot(&self) -> Window {
    unsafe {
      let window = XCreateSimpleWindow(self.display, self.root, 0, 0, self.get_display_width(0), self.get_display_height(0), 0, 0, 0);
      XMapWindow(self.display, window);
      window
    }
  }

  pub fn get_display_width(&self, screen: u32) -> u32 {
    unsafe {
      XDisplayWidth(self.display, screen as i32) as u32
    }
  }

  pub fn get_display_height(&self, screen: u32) -> u32 {
    unsafe {
      XDisplayHeight(self.display, screen as i32) as u32
    }
  }

  pub fn get_display_rect(&self, screen: u32) -> Rect {
    Rect{x: 0, y: 0, width: self.get_display_width(screen), height: self.get_display_height(screen)}
  }

  pub fn map_to_parent(&self, parent: Window, window: Window) {
    unsafe {
      XReparentWindow(self.display, window, parent, 0, 0);
      XMapWindow(self.display, window);
    }
  }

  pub fn configure_window(&mut self, window: Window, window_changes: WindowChanges, mask: u64) {
    unsafe {
      let mut ret_window_changes = XWindowChanges{
        x: window_changes.x as i32,
        y: window_changes.y as i32,
        width: window_changes.width as i32,
        height: window_changes.height as i32,
        border_width: window_changes.border_width as i32,
        sibling: window_changes.sibling,
        stack_mode: window_changes.stack_mode as i32
      };
      XConfigureWindow(self.display, window, mask as u32, &mut ret_window_changes);
    }
}

  pub fn move_window(&self, window: Window, x: u32, y: u32) {
    unsafe {
      XMoveWindow(self.display, window, x as i32, y as i32);
    }
  }

  pub fn resize_window(&self, window: Window, width: u32, height: u32) {
    unsafe {
      // TODO: the borderwidth should not be hardcoded
      XResizeWindow(self.display, window, width as u32 - 2, height as u32 - 2);
    }
  }

  pub fn move_resize_window(&self, window: Window, x: u32, y: u32, width: u32, height: u32) {
    unsafe {
      XMoveResizeWindow(self.display, window, x as i32, y as i32, width, height);
    }
  }

  pub fn raise_window(&self, window: Window) {
    unsafe {
      XRaiseWindow(self.display, window);
    }
  }

  pub fn set_window_border_width(&self, window: Window, width: u32) {
    if window != self.root {
      unsafe {
        XSetWindowBorderWidth(self.display, window, width);
      }
    }
  }

  pub fn set_window_border_color(&self, window: Window, color: u64) {
    if window != self.root {
      unsafe {
        XSetWindowBorder(self.display, window, color);
      }
    }
  }

  pub fn setup_window(&self, x: u32, y: u32, width: u32, height: u32, vroot: Window, window: Window) {
    unsafe {
      XSelectInput(self.display, window, 0x020031);
    }

    self.set_window_border_width(window, 1);
    self.set_window_border_color(window, 0x00FF0000);

    self.map_to_parent(vroot, window);
    self.move_resize_window(window, x, y, width, height);
  }

  fn get_window_name(&self, window: Window) -> String {
    if window == self.root {
      return String::from_str("root");
    }

    unsafe {
      let mut name : *mut c_char = uninitialized();
      XFetchName(self.display, window, &mut name);
      String::from_str(c_str_to_static_slice(transmute(name)))
    }
  }

  fn cast_event_to<T>(&self) -> &T {
    unsafe {
      &*(self.event as *const T)
    }
  }

  pub fn get_event(&self) -> XlibEvent {
    unsafe {
      XNextEvent(self.display, self.event);
    }

    let evt_type : c_int = *self.cast_event_to();
    match evt_type{
      MapRequest => {
        let evt : &XMapRequestEvent = self.cast_event_to();
        XMapRequest(evt.window)
      },
      ConfigureRequest => {
        let event : &XConfigureRequestEvent = self.cast_event_to();
        let changes = WindowChanges{
          x: event.x as u32,
          y: event.y as u32,
          width: event.width as u32,
          height: event.height as u32,
          border_width: event.border_width as u32,
          sibling: event.above as Window,
          stack_mode: event.detail as u32
        };
        XConfigurationRequest(event.window, changes, event.value_mask)
      },
      DestroyNotify => {
        let evt : &XDestroyWindowEvent = self.cast_event_to();
        XDestroyNotify(evt.window)
      },
      EnterNotify => {
        let evt: &XEnterWindowEvent = self.cast_event_to();
        XEnterNotify(evt.window, evt.detail)
      },
      LeaveNotify => {
        let evt: &XLeaveWindowEvent = self.cast_event_to();
        XLeaveNotify(evt.window, evt.detail)
      },
      KeyPress => {
        let evt: &XKeyPressedEvent = self.cast_event_to();
        XKeyPress(evt.window, evt.state, evt.keycode)
      }
      _ => {
        Unknown
      }
    }
  }
}