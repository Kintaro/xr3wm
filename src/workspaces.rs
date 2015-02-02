#![allow(dead_code, unused_must_use)]

use config::Config;
use container::Container;
use layout::{Layout, MoveOp, HSplitLayout, VSplitLayout};
use xlib::Window;
use xlib_window_system::XlibWindowSystem;

pub struct WorkspaceConfig<'a> {
  pub tag: String,
  pub screen: usize,
  pub layout: Box<Layout + 'a>
}

struct Workspace<'a> {
  managed: Container<'a>,
  unmanaged: Vec<Window>,
  tag: String,
  screen: usize,
  visible: bool,
  focus: Window
}

impl<'a> Workspace<'a> {
  pub fn new(tag: String, screen: usize, layout: Box<Layout + 'a>) -> Workspace<'a> {
    Workspace {
      managed: Container::new(layout.copy()),
      unmanaged: Vec::new(),
      tag: tag.clone(),
      screen: screen,
      visible: false,
      focus: 0
    }
  }

  pub fn contains(&self, window: Window) -> bool {
    self.is_unmanaged(window) || self.managed.contains_rec(window)
  }

  pub fn is_unmanaged(&self, window: Window) -> bool {
    self.unmanaged.iter().any(|&x| x == window)
  }

  pub fn focus(&self, ws: &XlibWindowSystem, config: &Config) {
    if self.focus != 0 {
      ws.focus_window(self.focus, config.border_focus_color);
      ws.skip_enter_events();
    }
  }

  pub fn add_window(&mut self, ws: &XlibWindowSystem, config: &Config, window: Window) {
    if self.focus != 0 {
      if let Some((_, c)) = self.managed.find_window(self.focus) {
        c.add_window(window);
      }
    } else {
      self.managed.add_window(window);
    }

    self.focus = window;
    ws.show_window(window);
    self.redraw(ws, config);
  }

  pub fn add_container(&mut self, index: usize) {
    if self.focus != 0 {
      if let Some((_, c)) = self.managed.find_window(self.focus) {
        c.nest_container(if index == 0 { HSplitLayout::new() } else { VSplitLayout::new() });
      }
    } else {
      self.managed.nest_container(if index == 0 { HSplitLayout::new() } else { VSplitLayout::new() });
    }
  }

  pub fn move_focus(&mut self, ws: &XlibWindowSystem, config: &Config, op: MoveOp) {
    if self.focus == 0 {
      return;
    }
    
    if let Some((_,container)) = self.managed.find_window(self.focus) {
      ws.set_window_border_color(self.focus, config.border_color);
      self.focus = container.move_focus(op);
    }
    self.focus(ws, config);
  }

  pub fn move_window(&mut self, ws: &XlibWindowSystem, config: &Config, op: MoveOp) {
    /*
    if self.focus == 0 {
      return;
    }

    let old_focus = self.focus;
    let mut new_focus = 0;
    if let Some((index,container)) = self.managed.find_window(self.focus) {
      //let index = self.managed.index_of_window(self.focus);
      new_focus = container.move_focus(op);
      if new_focus == self.focus {
        return;
      }

      //ws.set_window_border_color(self.focus, config.border_color);
      container.add_window(new_focus);
      container.remove_at(index);
    }

    if let Some((index,container)) = self.managed.find_window(new_focus) {
      container.add_window(old_focus);
      container.remove_at(index);
    }
    self.focus = 0;*/

    if let Some((_,container)) = self.managed.find_window(self.focus) {
      container.move_window(op);
    }

    self.redraw(ws, config);
  }

  fn redraw(&self, ws: &XlibWindowSystem, config: &Config) {
    debug!("Redraw...");

    let screen = ws.get_screen_infos()[self.screen];

    for &(window, rect) in self.managed.apply_layout(ws, screen).iter() {
      debug!("  {}, {:?}", window, rect);
      ws.setup_window(rect.x, rect.y, rect.width, rect.height, config.border_width, config.border_color, window);
    }

    self.focus(ws, config);
/*
    for &window in self.unmanaged.visible.iter() {
      let mut rect = ws.get_geometry(window);
      rect.width = rect.width + (2 * config.border_width);
      rect.height = rect.height + (2 * config.border_width);

      ws.setup_window((screen.width - rect.width) / 2, (screen.height - rect.height) / 2, rect.width, rect.height, config.border_width, config.border_color, window);
    }

    for &window in self.all_urgent().iter() {
      ws.set_window_border_color(window, config.border_urgent_color);
    }

    self.focus(ws, config);*/
  }
}

pub struct Workspaces<'a> {
  list: Vec<Workspace<'a>>,
  curr: usize
}

impl<'a> Workspaces<'a> {
  pub fn new<'b>(config: &'b Config<'a>, screens: usize) -> Workspaces<'a> {
    let mut workspaces = Workspaces {
      list: config.workspaces.iter().map(|c| {
        Workspace::new(c.tag.clone(), c.screen, c.layout.copy())
      }).collect(),
      curr: 0,
    };

    for screen in range(0us, screens) {
      if workspaces.list.iter().find(|ws| ws.screen == screen).is_none() {
        match workspaces.list.iter_mut().filter(|ws| ws.screen == 0).nth(1) {
          Some(ws) => {
            ws.screen = screen;
          },
          None => {}
        }
      }
    }

    for screen in range(0us, screens) {
      let ws = workspaces.list.iter_mut().find(|ws| ws.screen == screen).unwrap();
      ws.visible = true;
    }

    workspaces
  }

  pub fn curr(&self) -> &Workspace<'a> {
    &self.list[self.curr]
  }

  pub fn curr_mut(&mut self) -> &mut Workspace<'a> {
    &mut self.list[self.curr]
  }

  pub fn is_unmanaged(&self, window: Window) -> bool {
    self.list.iter().any(|ws| ws.is_unmanaged(window))
  }

  pub fn contains(&self, window: Window) -> bool {
    self.list.iter().any(|ws| ws.contains(window))
  }
}

/*
use config::Config;
use layout::Layout;
use xlib::Window;
use xlib_window_system::XlibWindowSystem;
use self::MoveOp::*;
use std::old_io::{fs, BufferedReader, File};
use std::old_io::fs::PathExtensions;
use std::cmp;

struct Stack {
  hidden: Vec<Window>,
  visible: Vec<Window>,
  urgent: Vec<Window>,
  focused_window: Window
}

impl Stack {
  fn new() -> Stack {
    Stack {
      hidden: Vec::new(),
      visible: Vec::new(),
      urgent: Vec::new(),
      focused_window: 0
    }
  }

  fn all(&self) -> Vec<Window> {
    self.hidden.iter().chain(self.visible.iter()).map(|&x| x).collect()
  }

  fn len(&self) -> usize {
    self.hidden.len() + self.visible.len()
  }

  fn contains(&self, window: Window) -> bool {
    self.all().iter().any(|&x| x == window)
  }

  fn is_urgent(&self) -> bool {
    !self.urgent.is_empty()
  }

  fn index_of(&self, window: Window) -> usize {
    self.all().iter().enumerate().find(|&(_,&w)| w == window).map(|(i,_)| i).unwrap()
  }

  fn index_of_visible(&self, window: Window) -> usize {
    self.visible.iter().enumerate().find(|&(_,&w)| w == window).map(|(i,_)| i).unwrap()
  }

  fn index_of_hidden(&self, window: Window) -> usize {
    self.hidden.iter().enumerate().find(|&(_,&w)| w == window).map(|(i,_)| i).unwrap()
  }

  fn hide(&mut self, window: Window) {
    let index = self.index_of_visible(window);
    self.visible.remove(index);
    self.hidden.push(window);
  }

  fn remove(&mut self, index: usize) {
    if index < self.hidden.len() {
      self.hidden.remove(index);
    } else {
      self.visible.remove(index - self.hidden.len());
    }
  }
}

pub struct WorkspaceConfig<'a> {
  pub tag: String,
  pub screen: usize,
  pub layout: Box<Layout + 'a>
}

struct Workspace<'a> {
  managed: Stack,
  unmanaged: Stack,
  tag: String,
  screen: usize,
  visible: bool,
  layout: Box<Layout + 'a>
}

pub enum MoveOp {
  Up,
  Down,
  Swap
}

impl<'a> Workspace<'a> {
  pub fn add_window(&mut self, ws: &XlibWindowSystem, config: &Config, window: Window) {
    if !ws.is_window_floating(window) {
      debug!("Add Managed: {}", window);
      self.managed.visible.push(window);

      if self.unmanaged.len() > 0 {
        debug!("Restacking");
        ws.restack_windows(self.all());
      }
    } else {
      self.unmanaged.visible.push(window);
      debug!("Add Unmanaged: {}", window);
    }

    self.focus_window(ws, config, window);
    if self.visible {
      self.redraw(ws, config);
      ws.show_window(window);
    }
  }

  pub fn serialize(&self) -> String {
    format!("{}:{}:{}:{}:{}", self.screen, self.visible, self.managed.focused_window, self.unmanaged.focused_window, vec![
      self.managed.visible.iter().map(|&x| x.to_string()).collect::<Vec<String>>().connect(","),
      self.managed.hidden.iter().map(|&x| x.to_string()).collect::<Vec<String>>().connect(","),
      self.unmanaged.visible.iter().map(|&x| x.to_string()).collect::<Vec<String>>().connect(","),
      self.unmanaged.hidden.iter().map(|&x| x.to_string()).collect::<Vec<String>>().connect(","),
    ].connect(":").as_slice())
  }

  fn all(&self) -> Vec<Window> {
    self.unmanaged.all().iter().chain(self.managed.all().iter()).map(|&x| x).collect()
  }

  fn all_visible(&self) -> Vec<Window> {
    self.unmanaged.visible.iter().chain(self.managed.visible.iter()).map(|&x| x).collect()
  }

  fn all_urgent(&self) -> Vec<Window> {
    self.unmanaged.urgent.iter().chain(self.managed.urgent.iter()).map(|&x| x).collect()
  }

  pub fn get_layout(&self) -> &Box<Layout + 'a> {
    &self.layout
  }

  pub fn get_layout_mut(&mut self) -> &mut Box<Layout + 'a> {
    &mut self.layout
  }

  pub fn get_tag(&self) -> String {
    self.tag.clone()
  }

  pub fn is_unmanaged(&self, window: Window) -> bool {
    self.unmanaged.contains(window)
  }

  pub fn is_managed(&self, window: Window) -> bool {
    self.managed.contains(window)
  }

  pub fn is_urgent(&self) -> bool {
    self.managed.is_urgent() || self.unmanaged.is_urgent()
  }

  pub fn is_visible(&self) -> bool {
    self.visible
  }

  pub fn focused_window(&self) -> Window {
    if self.unmanaged.focused_window == 0 {
      self.managed.focused_window
    } else {
      self.unmanaged.focused_window
    }
  }

  pub fn set_urgency(&mut self, urgent: bool, ws: &XlibWindowSystem,config: &Config, window: Window) {
    if self.all_urgent().contains(&window) && !urgent {
      debug!("unset urgent {}", window);
      self.remove_urgent_window(window)
    } else {
      debug!("set urgent {}", window);
      if self.is_managed(window) {
        self.managed.urgent.push(window);
      } else {
        self.unmanaged.urgent.push(window);
      }
    }

    self.redraw(ws, config);
  }

  fn remove_urgent_window(&mut self, window: Window) {
    let index = self.all_urgent().iter().enumerate().find(|&(_,&x)| x == window).map(|(i,_)| i).unwrap();
    if index < self.unmanaged.urgent.len() {
      self.unmanaged.urgent.remove(index - 1);
    } else {
      self.managed.urgent.remove(self.unmanaged.urgent.len() - index);
    }
  }

  fn remove_managed(&mut self, ws: &XlibWindowSystem, config: &Config, window: Window) {
    let index = self.managed.index_of_visible(window);

    self.managed.focused_window = 0;
    ws.unmap_window(window);
    self.managed.remove(index);

    let new_focused_window = if !self.managed.visible.is_empty() {
      self.managed.visible[if index < self.managed.visible.len() { index } else { index - 1}]
    } else {
      0
    };

    if self.visible {
      self.redraw(ws, config);
      self.focus_window(ws, config, new_focused_window);
    }
  }

  fn remove_unmanaged(&mut self, ws: &XlibWindowSystem, config: &Config, window: Window) {
    let index = self.unmanaged.index_of_visible(window);

    self.unmanaged.focused_window = 0;
    ws.unmap_window(window);
    self.unmanaged.remove(index);

    let new_focused_window = if !self.unmanaged.visible.is_empty() {
      self.unmanaged.visible[if index < self.unmanaged.visible.len() { index } else { index - 1}]
    } else {
      if !self.managed.visible.is_empty() {
        self.managed.visible[self.managed.len() - 1]
      } else {
        0
      }
    };

    if self.visible {
      self.redraw(ws, config);
      self.focus_window(ws, config, new_focused_window);
    }
  }

  pub fn remove_window(&mut self, ws: &XlibWindowSystem, config: &Config, window: Window) {
    if self.managed.contains(window) {
      debug!("Remove Managed: {}", window);
      self.remove_managed(ws, config, window);
    } else {
      if self.unmanaged.contains(window) {
        debug!("Remove Unmanaged: {}", window);
        self.remove_unmanaged(ws, config, window);
      }
    }
  }

  pub fn hide_window(&mut self, window: Window) {
    if self.managed.contains(window) {
      self.managed.hide(window);
    } else {
      self.unmanaged.hide(window);
    }
  }

  pub fn focus_window(&mut self, ws: &XlibWindowSystem, config: &Config, window: Window) {
    if window == 0 {
      return;
    }

    self.unfocus_window(ws, config);

    if self.unmanaged.contains(window) {
      self.unmanaged.focused_window = window;
    } else {
      self.managed.focused_window = window;
    }

    ws.focus_window(window, config.border_focus_color);
  }

  pub fn unfocus_window(&mut self, ws: &XlibWindowSystem, config: &Config) {
    let focused_window = self.focused_window();

    if focused_window != 0 {
      ws.set_window_border_color(focused_window, config.border_color);
      
      if self.unmanaged.focused_window == 0 {
        self.managed.focused_window = 0;
      } else {
        self.unmanaged.focused_window = 0;
      }
    }
  }

  pub fn move_focus(&mut self, ws: &XlibWindowSystem, config: &Config, op: MoveOp) {
    let windows : Vec<Window> = self.all_visible();
    let count = windows.len();

    if self.focused_window() == 0 || count < 2 {
      return;
    }

    let index = windows.iter().enumerate().find(|&(_,&w)| w == self.focused_window()).map(|(i,_)| i).unwrap();
    let new_focused_window = match op {
      Up => {
        if index == 0 {
          windows[count - 1]
        } else {
          windows[index - 1]
        }
      },
      Down => {
        windows[(index + 1) % count]
      },
      Swap => {
        windows[0]
      }
    };

    self.focus_window(ws, config, new_focused_window);
  }

  pub fn move_window(&mut self, ws: &XlibWindowSystem, config: &Config, op: MoveOp) {
    let focused_window = self.focused_window();

    if focused_window == 0 || self.unmanaged.focused_window != 0 {
      return;
    }

    let pos = self.managed.index_of_visible(focused_window);
    let new_pos = match op {
      Up => {
        if pos == 0 {
          self.managed.visible.len() - 1
        } else {
          pos - 1
        }
      },
      Down => {
        (pos + 1) % self.managed.visible.len()
      },
      Swap => {
        let master = self.managed.visible[0];
        self.managed.visible.insert(pos, master);
        self.managed.visible.remove(0);
        0
      }
    };

    self.managed.visible.remove(pos);
    self.managed.visible.insert(new_pos, focused_window);

    self.redraw(ws, config);
  }

  pub fn contains(&self, window: Window) -> bool {
    self.all().iter().any(|&w| w == window)
  }

  pub fn unfocus(&mut self, ws: &XlibWindowSystem, config: &Config) {
    ws.set_window_border_color(self.focused_window(), config.border_color);
  }

  pub fn focus(&self, ws: &XlibWindowSystem, config: &Config) {
    if self.focused_window() != 0 {
      ws.focus_window(self.focused_window(), config.border_focus_color);
      ws.skip_enter_events();
    }
  }

  pub fn hide(&mut self, ws: &XlibWindowSystem) {
    self.visible = false;

    for &w in self.managed.visible.iter() {
      ws.hide_window(w);
    }

    for &w in self.unmanaged.visible.iter() {
      ws.hide_window(w);
    }
  }

  pub fn show(&mut self, ws: &XlibWindowSystem, config: &Config) {
    self.visible = true;

    self.redraw(ws, config);
    for &w in self.managed.visible.iter() {
      ws.show_window(w);
    }

    for &w in self.unmanaged.visible.iter() {
      ws.show_window(w);
    }
  }

  pub fn redraw(&self, ws: &XlibWindowSystem, config: &Config) {
    debug!("Redraw...");

    let screen = ws.get_screen_infos()[self.screen];

    for (i,rect) in self.layout.apply(ws, screen, &self.managed.visible).iter().enumerate() {
      debug!("  {}, {:?}", self.managed.visible[i], rect);
      ws.setup_window(rect.x, rect.y, rect.width, rect.height, config.border_width, config.border_color, self.managed.visible[i]);
    }

    for &window in self.unmanaged.visible.iter() {
      let mut rect = ws.get_geometry(window);
      rect.width = rect.width + (2 * config.border_width);
      rect.height = rect.height + (2 * config.border_width);

      ws.setup_window((screen.width - rect.width) / 2, (screen.height - rect.height) / 2, rect.width, rect.height, config.border_width, config.border_color, window);
    }

    for &window in self.all_urgent().iter() {
      ws.set_window_border_color(window, config.border_urgent_color);
    }

    self.focus(ws, config);
  }
}

pub struct Workspaces<'a> {
  list: Vec<Workspace<'a>>,
  cur: usize
}

impl<'a> Workspaces<'a> {
  pub fn new<'b>(config: &'b Config<'a>, screens: usize) -> Workspaces<'a> {
    if Path::new(concat!(env!("HOME"), "/.xr3wm/.tmp")).exists() {
      println!("recompiling... done");
      Workspaces::load_workspaces(config)
    } else {
      let mut workspaces = Workspaces {
        list: config.workspaces.iter().map(|c| {
          Workspace {
            managed: Stack::new(),
            unmanaged: Stack::new(),
            tag: c.tag.clone(),
            screen: c.screen,
            visible: false,
            layout: c.layout.copy()
          }
        }).collect(),
        cur: 0,
      };

      for screen in range(0us, screens) {
        if workspaces.list.iter().find(|ws| ws.screen == screen).is_none() {
          match workspaces.list.iter_mut().filter(|ws| ws.screen == 0).nth(1) {
            Some(ws) => {
              ws.screen = screen;
            },
            None => {}
          }
        }
      }

      for screen in range(0us, screens) {
        let ws = workspaces.list.iter_mut().find(|ws| ws.screen == screen).unwrap();
        ws.visible = true;
      }

      workspaces
    }
  }

  fn load_workspaces(config: &Config) -> Workspaces<'a>{
    let path = Path::new(concat!(env!("HOME"), "/.xr3wm/.tmp"));
    let mut file = BufferedReader::new(File::open(&path));
    let cur = file.read_line().unwrap();
    let lines : Vec<String> = file.lines().map(|x| x.unwrap()).collect();
    fs::unlink(&path);

    Workspaces {
      list: config.workspaces.iter().enumerate().map(|(i,c)| {
        if i < lines.len() {
          let data : Vec<&str> = lines.get(i).unwrap().as_slice().split(':').collect();

          let mut managed = Stack::new();
          let mut unmanaged = Stack::new();

          managed.focused_window = data[2].parse::<u64>().unwrap();
          unmanaged.focused_window = data[3].parse::<u64>().unwrap();
          managed.visible = data[4].split(',').filter_map(|x| x.parse::<u64>()).collect();
          managed.hidden = data[5].split(',').filter_map(|x| x.parse::<u64>()).collect();
          unmanaged.visible = data[6].split(',').filter_map(|x| x.parse::<u64>()).collect();
          unmanaged.hidden = data[7].split(',').filter_map(|x| x.parse::<u64>()).collect();
          debug!("loading workspace {}", i+1);

          Workspace {
            managed: managed,
            unmanaged: unmanaged,
            tag: c.tag.clone(),
            screen: data[0].parse::<usize>().unwrap(),
            visible: data[1].parse::<bool>().unwrap(),
            layout: c.layout.copy()
          }
        } else {
          Workspace {
            managed: Stack::new(),
            unmanaged: Stack::new(),
            tag: c.tag.clone(),
            screen: c.screen,
            visible: false,
            layout: c.layout.copy()
          }
        }
      }).collect(),
    cur: cur.slice_to(cur.len()-1).parse::<usize>().unwrap()
    }
  }

  pub fn serialize(&self) -> String {
    format!("{}\n{}", self.cur, self.list.iter().map(|x| x.serialize()).collect::<Vec<String>>().connect("\n"))
  }

  pub fn get(&self, index: usize) -> &Workspace<'a> {
    if index < self.list.len() {
      self.list.get(index).unwrap()
    } else {
      self.current()
    }
  }

  pub fn get_mut(&mut self, index: usize) -> &mut Workspace<'a> {
    if index < self.list.len() {
      self.list.get_mut(index).unwrap()
    } else {
      self.current_mut()
    }
  }

  pub fn current(&self) -> &Workspace<'a> {
    self.list.get(self.cur).unwrap()
  }

  pub fn current_mut(&mut self) -> &mut Workspace<'a> {
    self.list.get_mut(self.cur).unwrap()
  }

  pub fn all(&self) -> &Vec<Workspace<'a>> {
    &self.list
  }

  pub fn get_index(&self) -> usize {
    self.cur
  }

  pub fn contains(&self, window: Window) -> bool {
    self.list.iter().any(|ws| ws.contains(window))
  }

  pub fn is_unmanaged(&self, window: Window) -> bool {
    self.list.iter().any(|ws| ws.is_unmanaged(window))
  }

  pub fn focus_window(&mut self, ws: &XlibWindowSystem, config: &Config, window: Window) {
    match self.list.iter().enumerate().find(|&(_,workspace)| workspace.contains(window)).map(|(i,_)| i) {
      Some(index) => {
        if self.cur != index {
          self.list[index].focus_window(ws, config, window);
          self.switch_to(ws, config, index);
        } else {
          self.current_mut().focus_window(ws, config, window);
        }
      },
      None => {}
    }
  }

  pub fn switch_to(&mut self, ws: &XlibWindowSystem, config: &Config, index: usize) {
    if self.cur != index && index < self.list.len() {
      if self.list[index].visible {
        if config.greedy_view {
          self.switch_screens(index);
          self.list[self.cur].show(ws, config);
        }
      } else {
        self.list[index].screen = self.list[self.cur].screen;
        self.list[index].show(ws, config);
        self.list[self.cur].hide(ws);
      }

      self.list[self.cur].unfocus(ws, config);
      //self.list[index].show(ws, config);
      self.list[index].focus(ws, config);
      self.cur = index;
    }
  }

  pub fn switch_to_screen(&mut self, ws: &XlibWindowSystem, config: &Config, screen: usize) {
    match self.list.iter().enumerate().filter(|&(i,ws)| ws.screen == screen && ws.visible && i != self.cur).map(|(i,_)| i).last() {
      Some(index) => {
        self.list[self.cur].unfocus(ws, config);
        self.list[index].focus(ws, config);
        self.cur = index;
      },
      None => { }
    }
  }

  pub fn move_window_to(&mut self, ws: &XlibWindowSystem, config: &Config, index: usize) {
    let window = self.list[self.cur].focused_window();
    if window == 0 || index == self.cur {
      return;
    }

    self.remove_window(ws, config, window);
    self.list[index].add_window(ws, config, window);
    self.list[index].unfocus(ws, config);
  }

  pub fn move_window_to_screen(&mut self, ws: &XlibWindowSystem, config: &Config, screen: usize) {
    match self.list.iter().enumerate().find(|&(_,ws)| ws.screen == screen) {
      Some((index,_)) => {
        self.move_window_to(ws, config, index);
      },
      None => {}
    }
  }

  pub fn remove_window(&mut self, ws: &XlibWindowSystem, config: &Config, window: Window) {
    match self.find_window(window) {
      Some(workspace) => {
        workspace.remove_window(ws, config, window);
      },
      None => {}
    }
  }

  pub fn hide_window(&'a mut self, window: Window) {
    if let Some(workspace) = self.find_window(window) {
      workspace.hide_window(window);
    }
  }

  pub fn rescreen(&mut self, ws: &XlibWindowSystem, config: &Config) {
    let new_screens = ws.get_screen_infos().len();
    let prev_screens = self.list.iter().fold(0, |acc, x| cmp::max(acc, x.screen));
    debug!("rescreen {}", new_screens);

    // move and hide workspaces if their screens got removed
    for workspace in self.list.iter_mut().filter(|ws| ws.screen > (new_screens - 1)) {
      workspace.screen = 0;
      workspace.hide(ws);
    }

    // assign the first hidden workspace to the new screen
    for screen in range(prev_screens + 1, new_screens) {
      match self.list.iter_mut().find(|ws| !ws.visible) {
        Some(workspace) => {
          workspace.screen = screen;
          workspace.show(ws, config);
        },
        None => {
          break;
        }
      }
    }

    self.list.iter_mut().find(|ws| ws.screen == 0).unwrap().show(ws, config);
  }

  pub fn find_window(&mut self, window: Window) -> Option<&mut Workspace<'a>> {
    self.list.iter_mut().find(|workspace| workspace.contains(window))
  }

  fn switch_screens(&mut self, dest: usize) {
    let screen = self.list[self.cur].screen;
    self.list[self.cur].screen = self.list[dest].screen;
    self.list[dest].screen = screen;
  }
}
*/