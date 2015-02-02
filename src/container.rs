use xlib_window_system::XlibWindowSystem;
use xlib::Window;
use layout::{Layout, Rect, MoveOp};
use std::slice::{Iter, IterMut};
use std::ptr;
use std::mem;

struct Rawlink<'a> {
  p: *mut Container<'a>,
}

impl<'a> Copy for Rawlink<'a> {}
unsafe impl<'a> Send for Rawlink<'static> {}
unsafe impl<'a> Sync for Rawlink<'a> {}

impl<'a> Rawlink<'a> {
  fn none() -> Rawlink<'a> {
    Rawlink{p: ptr::null_mut()}
  }

  fn some(n: &mut Container<'a>) -> Rawlink<'a> {
    Rawlink{p: n}
  }

  fn resolve_immut<'b>(&self) -> Option<&'b Container<'a>> {
    unsafe {
      mem::transmute(self.p.as_ref())
   }
  }

  fn resolve<'b>(&mut self) -> Option<&'b mut Container<'a>> {
    if self.p.is_null() {
      None
    } else {
      Some(unsafe { mem::transmute(self.p) })
    }
  }

  fn take(&mut self) -> Rawlink<'a> {
    mem::replace(self, Rawlink::none())
  }
}

impl<'a> Clone for Rawlink<'a> {
    #[inline]
    fn clone(&self) -> Rawlink<'a> {
        Rawlink{p: self.p}
    }
}

pub enum Node<'a> {
  SplitContainer(Container<'a>),
  Window(Window)
}

#[experimental]
pub struct WindowIter<'a> {
  iter: Iter<'a, Node<'a>>,
  index: usize
}

impl<'a> Iterator for WindowIter<'a> {
  type Item = (usize, Window);

  #[inline]
  fn next(&mut self) -> Option<(usize, Window)> {
    if let Some(next) = self.iter.next() {
      let curr = self.index;
      self.index += 1;
      match *next {
        Node::Window(w) => {
          Some((curr, w))
        },
        _ => {
          self.next()
        }
      }
    } else {
      None
    }
  }
}

pub struct Container<'a> {
  parent: Rawlink<'a>,
  visible: Vec<Node<'a>>,
  hidden: Vec<Node<'a>>,
  layout: Box<Layout + 'a>,
  focus: usize
}

impl<'a> Container<'a> {
  pub fn new(layout: Box<Layout + 'a>) -> Container<'a> {
    Container {
      parent: Rawlink::none(),
      visible: Vec::new(),
      hidden: Vec::new(),
      layout: layout,
      focus: 0,
    }
  }

  pub fn iter(&'a self) -> WindowIter {
    WindowIter {
      iter: self.visible.iter(),
      index: 0
    }
  }

  fn windows(&self) -> Vec<(usize, Window)> {
    self.visible.iter().enumerate().filter_map(|(i,x)| if let &Node::Window(w) = x { Some((i,w)) } else { None }).collect()
  }

  fn containers<'b>(&'b self) -> Vec<&'b Container<'a>> {
    self.visible.iter().filter_map(|x| if let &Node::SplitContainer(ref w) = x { Some(w) } else { None }).collect()
  }

  fn containers_mut<'b>(&'b mut self) -> Vec<&'b mut Container<'a>> {
    self.visible.iter_mut().filter_map(|x| if let &mut Node::SplitContainer(ref mut w) = x { Some(w) } else { None }).collect()
  }

  fn parent<'b>(&self) -> Option<&'b Container<'a>> {
    self.parent.resolve_immut()
  }

  fn parent_mut<'b>(&mut self) -> Option<&'b mut Container<'a>> {
    self.parent.resolve()
  }

  pub fn count(&self) -> usize {
    self.visible.len()
  }

  pub fn contains(&self, window: Window) -> bool {
    self.windows().iter().any(|&(_,w)| w == window)
  }

  pub fn contains_rec(&self, window: Window) -> bool {
    self.contains(window) || self.containers().iter().any(|&c| c.contains_rec(window))
  }

  pub fn focused_window(&self) -> Window {
    match self.visible[self.focus] {
      Node::Window(w) => {
        w
      },
      Node::SplitContainer(ref c) => {
        c.focused_window()
      }
    }
  }

  pub fn index_of_window(&mut self, window: Window) -> usize {
    self.windows().iter().find(|&&(_,w)| w == window).map(|&(i,_)| i).unwrap()
  }

  pub fn find_window(&mut self, window: Window) -> Option<(usize, &mut Container<'a>)> {
    if let Some(&(i,_)) = self.windows().iter().find(|&&(_,w)| w == window) {
      Some((i,self))
    } else {
      for c in self.containers_mut().into_iter() {
        if let Some((i,container)) = c.find_window(window) {
          return Some((i,container));
        }
      }
      None
    }
  }

  fn add_node(&mut self, node: Node<'a>) {
    if self.visible.is_empty() {
      self.visible.push(node);
    } else {
      self.focus += 1;
      self.visible.insert(self.focus, node);
    }
    debug!("add focus: {}", self.focus);    
  }

  pub fn add_window(&mut self, window: Window) {
    self.add_node(Node::Window(window));
  }

  pub fn remove_window(&mut self, window: Window) {
    let index = self.index_of_window(window);
    debug!("remove: {}, {}, {}", self.focus, index, self.visible.len());
    self.visible.remove(index);
    if index != 0 && index >= self.focus {
      self.focus -= 1;
    }
    debug!("remove focus: {}", self.focus);
  }

  pub fn remove_at(&mut self, index: usize) {
    self.visible.remove(index);
    if index != 0 && index >= self.focus {
      self.focus -= 1;
    }
  }

  pub fn nest_container(&mut self, layout: Box<Layout + 'a>) {
    if !self.visible.is_empty() {
      let mut container = Container::new(layout);
      container.parent = Rawlink::some(self);

      container.visible.push(self.visible.remove(self.focus));
      self.visible.insert(self.focus, Node::SplitContainer(container));
    } else {
      self.layout = layout;
    }
  }

  pub fn move_focus(&mut self, op: MoveOp) -> Window {
    let mut parent = self.parent_mut();
    let (index, crossing) = self.layout.move_focus(self.focus, self.visible.len(), op.clone());
    
    if(crossing && parent.is_some()) {
      parent.unwrap().move_focus(op)
    } else {
      self.focus = index;
      self.focused_window()
    }
  }

  pub fn move_window(&mut self, op: MoveOp) {
    let mut parent = self.parent_mut();
    let (mut index, crossing) = self.layout.move_focus(self.focus, self.visible.len(), op.clone());

    if ((index == self.focus) && !crossing) || (crossing && parent.is_none()) {
      return;
    }
    
    let node = self.visible.remove(self.focus);
    if crossing {
      let old_index = parent.unwrap().focus;
      parent.unwrap().visible.insert(if index < old_index);
    } else {
      debug!("{}, {}", self.focus, index);
      
      if let &mut Node::SplitContainer(ref mut c) = self.visible.get_mut(if self.focus < index { index - 1 } else { index }).unwrap() {
        c.add_node(node);
        self.focus = index;
        return;
      }
      self.visible.insert(index, node);
      self.focus = index;
    }
  }

  pub fn apply_layout(&self, ws: &XlibWindowSystem, area: Rect) -> Vec<(Window, Rect)> {
    let mut ret : Vec<(Window, Rect)> = Vec::new();
    let rects = self.layout.apply(ws, area, &self.visible);

    for (i,node) in self.visible.iter().enumerate() {
      match *node {
        Node::Window(w) => {
          ret.push((w, rects[i]));
        },
        Node::SplitContainer(ref c) => {
          for &(w,r) in c.apply_layout(ws, rects[i]).iter() {
            ret.push((w, r));
          }
        }
      }
    }
    ret
  }
}