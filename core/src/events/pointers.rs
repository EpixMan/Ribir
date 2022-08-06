use super::EventCommon;
use crate::{impl_query_self_only, prelude::*};
use std::time::{Duration, Instant};

mod from_mouse;
#[derive(Debug, Clone)]
pub struct PointerId(usize);

/// The pointer is a hardware-agnostic device that can target a specific set of
/// screen coordinates. Having a single event model for pointers can simplify
/// creating Web sites and applications and provide a good user experience
/// regardless of the user's hardware. However, for scenarios when
/// device-specific handling is desired, pointer events defines a pointerType
/// property to inspect the device type which produced the event.
/// Reference: <https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events#term_pointer_event>
#[derive(Debug, Clone)]
pub struct PointerEvent {
  /// A unique identifier for the pointer causing the event.
  pub id: PointerId,
  /// The width (magnitude on the X axis), in pixels, of the contact geometry of
  /// the pointer.
  pub width: f32,
  /// the height (magnitude on the Y axis), in pixels, of the contact geometry
  /// of the pointer.
  pub height: f32,
  /// the normalized pressure of the pointer input in the range of 0 to 1, where
  /// 0 and 1 represent the minimum and maximum pressure the hardware is capable
  /// of detecting, respectively. tangentialPressure
  /// The normalized tangential pressure of the pointer input (also known as
  /// barrel pressure or cylinder stress) in the range -1 to 1, where 0 is the
  /// neutral position of the control.
  pub pressure: f32,
  /// The plane angle (in degrees, in the range of -90 to 90) between the Y–Z
  /// plane and the plane containing both the pointer (e.g. pen stylus) axis and
  /// the Y axis.
  pub tilt_x: f32,
  /// The plane angle (in degrees, in the range of -90 to 90) between the X–Z
  /// plane and the plane containing both the pointer (e.g. pen stylus) axis and
  /// the X axis.
  pub tilt_y: f32,
  /// The clockwise rotation of the pointer (e.g. pen stylus) around its major
  /// axis in degrees, with a value in the range 0 to 359.
  pub twist: f32,
  ///  Indicates the device type that caused the event (mouse, pen, touch, etc.)
  pub point_type: PointerType,
  /// Indicates if the pointer represents the primary pointer of this pointer
  /// type.
  pub is_primary: bool,

  pub common: EventCommon,
}

bitflags! {
  #[derive(Default)]
  pub struct MouseButtons: u8 {
    /// Primary button (usually the left button)
    const PRIMARY = 0b0000_0001;
    /// Secondary button (usually the right button)
    const SECONDARY = 0b0000_0010;
    /// Auxiliary button (usually the mouse wheel button or middle button)
    const AUXILIARY = 0b0000_0100;
    /// 4th button (typically the "Browser Back" button)
    const FOURTH = 0b0000_1000;
    /// 5th button (typically the "Browser Forward" button)
    const FIFTH = 0b0001_0000;
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerType {
  /// The event was generated by a mouse device.
  Mouse,
  /// The event was generated by a pen or stylus device.
  Pen,
  /// The event was generated by a touch, such as a finger.
  Touch,
}

impl std::borrow::Borrow<EventCommon> for PointerEvent {
  #[inline]
  fn borrow(&self) -> &EventCommon { &self.common }
}

impl std::borrow::BorrowMut<EventCommon> for PointerEvent {
  #[inline]
  fn borrow_mut(&mut self) -> &mut EventCommon { &mut self.common }
}

impl std::ops::Deref for PointerEvent {
  type Target = EventCommon;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.common }
}

impl std::ops::DerefMut for PointerEvent {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.common }
}

macro_rules! impl_pointer_listener {
  ($name: ident, $field: ident, $convert: ident, $builder: ident) => {
    #[derive(Declare)]
    pub struct $name {
      #[declare(builtin, custom_convert)]
      pub $field: Box<dyn for<'r> FnMut(&'r mut PointerEvent)>,
    }

    impl $builder {
      #[inline]
      pub fn $convert(
        f: impl for<'r> FnMut(&'r mut PointerEvent) + 'static,
      ) -> Box<dyn for<'r> FnMut(&'r mut PointerEvent)> {
        Box::new(f)
      }
    }

    impl ComposeSingleChild for $name {
      fn compose_single_child(
        this: Stateful<Self>,
        child: Option<Widget>,
        _: &mut BuildCtx,
      ) -> Widget {
        compose_child_as_data_widget(child, this, |w| w)
      }
    }

    impl Query for $name {
      impl_query_self_only!();
    }

    impl EventListener for $name {
      type Event = PointerEvent;
      #[inline]
      fn dispatch(&mut self, event: &mut PointerEvent) { (self.$field)(event) }
    }
  };
}

impl_pointer_listener!(
  PointerDownListener,
  on_pointer_down,
  on_pointer_down_convert,
  PointerDownListenerBuilder
);

impl_pointer_listener!(
  PointerUpListener,
  on_pointer_up,
  on_pointer_up_convert,
  PointerUpListenerBuilder
);

impl_pointer_listener!(
  PointerMoveListener,
  on_pointer_move,
  on_pointer_move_convert,
  PointerMoveListenerBuilder
);

impl_pointer_listener!(TapListener, on_tap, on_tap_convert, TapListenerBuilder);

impl_pointer_listener!(
  PointerCancelListener,
  on_pointer_cancel,
  on_pointer_cancel_convert,
  PointerCancelListenerBuilder
);

impl_pointer_listener!(
  PointerEnterListener,
  on_pointer_enter,
  on_pointer_enter_convert,
  PointerEnterListenerBuilder
);

impl_pointer_listener!(
  PointerLeaveListener,
  on_pointer_leave,
  on_pointer_leave_convert,
  PointerLeaveListenerBuilder
);

#[derive(Declare)]
pub struct XTimesTapListener {
  #[declare(custom_convert, builtin)]
  pub on_x_times_tap: (u8, Box<dyn for<'r> FnMut(&'r mut PointerEvent)>),
}

#[derive(Declare)]
pub struct DoubleTapListener {
  #[declare(custom_convert, builtin)]
  pub on_double_tap: Box<dyn for<'r> FnMut(&'r mut PointerEvent)>,
}

#[derive(Declare)]
pub struct TripleTapListener {
  #[declare(custom_convert, builtin)]
  pub on_tripe_tap: Box<dyn for<'r> FnMut(&'r mut PointerEvent)>,
}

impl ComposeSingleChild for XTimesTapListener {
  fn compose_single_child(this: Stateful<Self>, child: Option<Widget>, _: &mut BuildCtx) -> Widget {
    let on_tap: Box<dyn for<'r> FnMut(&'r mut PointerEvent)> = match this.try_into_stateless() {
      Ok(w) => {
        let (times, on_x_times_tap) = w.on_x_times_tap;
        Box::new(TapListener::on_tap_times(times, on_x_times_tap))
      }
      Err(this) => {
        let times = this.shallow_ref().on_x_times_tap.0;
        let handler =
          TapListener::on_tap_times(times, move |e| (this.state_ref().on_x_times_tap.1)(e));
        Box::new(handler)
      }
    };
    widget! {
      ExprWidget { expr: child, on_tap }
    }
  }
}

impl ComposeSingleChild for DoubleTapListener {
  fn compose_single_child(this: Stateful<Self>, child: Option<Widget>, _: &mut BuildCtx) -> Widget {
    let on_tap: Box<dyn for<'r> FnMut(&'r mut PointerEvent)> = match this.try_into_stateless() {
      Ok(w) => Box::new(TapListener::on_tap_times(2, w.on_double_tap)),
      Err(this) => {
        let handler = TapListener::on_tap_times(2, move |e| (this.state_ref().on_double_tap)(e));
        Box::new(handler)
      }
    };
    widget! {
      ExprWidget { expr: child, on_tap }
    }
  }
}

impl ComposeSingleChild for TripleTapListener {
  fn compose_single_child(this: Stateful<Self>, child: Option<Widget>, _: &mut BuildCtx) -> Widget {
    let on_tap: Box<dyn for<'r> FnMut(&'r mut PointerEvent)> = match this.try_into_stateless() {
      Ok(w) => Box::new(TapListener::on_tap_times(3, w.on_tripe_tap)),
      Err(this) => {
        let handler = TapListener::on_tap_times(3, move |e| (this.state_ref().on_tripe_tap)(e));
        Box::new(handler)
      }
    };
    widget! {
      ExprWidget { expr: child, on_tap }
    }
  }
}

impl XTimesTapListenerBuilder {
  #[inline]
  pub fn on_x_times_tap_convert(
    f: (u8, impl for<'r> FnMut(&'r mut PointerEvent) + 'static),
  ) -> (u8, Box<dyn for<'r> FnMut(&'r mut PointerEvent)>) {
    (f.0, Box::new(f.1))
  }
}

impl DoubleTapListener {
  #[inline]
  pub fn on_double_tap_convert(
    f: impl for<'r> FnMut(&'r mut PointerEvent) + 'static,
  ) -> Box<dyn for<'r> FnMut(&'r mut PointerEvent)> {
    Box::new(f)
  }
}

impl TripleTapListener {
  #[inline]
  pub fn on_triple_tap_convert(
    f: impl for<'r> FnMut(&'r mut PointerEvent) + 'static,
  ) -> Box<dyn for<'r> FnMut(&'r mut PointerEvent)> {
    Box::new(f)
  }
}

impl TapListener {
  pub fn on_tap_times<H: FnMut(&mut PointerEvent) + 'static>(
    times: u8,
    mut handler: H,
  ) -> impl FnMut(&mut PointerEvent) + 'static {
    const DUR: Duration = Duration::from_millis(250);
    #[derive(Clone)]
    struct TapInfo {
      first_tap_stamp: Instant,
      tap_times: u8,
      pointer_type: PointerType,
      mouse_btns: MouseButtons,
    }
    let mut tap_info: Option<TapInfo> = None;
    move |e| {
      match &mut tap_info {
        Some(info)
          if info.pointer_type == e.point_type
            && info.mouse_btns == e.mouse_buttons()
            && info.tap_times < times
            && info.first_tap_stamp.elapsed() < DUR =>
        {
          info.tap_times += 1;
        }
        _ => {
          tap_info = Some(TapInfo {
            first_tap_stamp: Instant::now(),
            tap_times: 1,
            pointer_type: e.point_type.clone(),
            mouse_btns: e.mouse_buttons(),
          })
        }
      };

      let info = tap_info.as_mut().unwrap();
      if info.tap_times == times {
        info.tap_times = 0;
        handler(e)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use futures::executor::LocalPool;
  use std::{cell::RefCell, rc::Rc};
  use winit::event::{DeviceId, ElementState, ModifiersState, MouseButton, WindowEvent};

  fn env(times: u8) -> (Window, Rc<RefCell<usize>>) {
    let size = Size::new(400., 400.);
    let count = Rc::new(RefCell::new(0));
    let c_count = count.clone();
    let w = widget! {
      SizedBox {
        size,
        on_x_times_tap: (times, move |_| *c_count.borrow_mut() += 1)
      }
    };
    let mut wnd = Window::without_render(w, size);
    wnd.draw_frame();

    (wnd, count)
  }

  #[test]
  fn double_tap() {
    let (mut wnd, count) = env(2);

    let mut local_pool = LocalPool::new();
    let device_id = unsafe { DeviceId::dummy() };
    observable::interval(Duration::from_millis(10), local_pool.spawner())
      .take(8)
      .subscribe(move |i| {
        wnd.processes_native_event(WindowEvent::MouseInput {
          device_id,
          state: if i % 2 == 0 {
            ElementState::Pressed
          } else {
            ElementState::Released
          },
          button: MouseButton::Left,
          modifiers: ModifiersState::default(),
        });
      });

    local_pool.run();

    assert_eq!(*count.borrow(), 2);

    let (mut wnd, count) = env(2);
    observable::interval(Duration::from_millis(251), local_pool.spawner())
      .take(8)
      .subscribe(move |i| {
        wnd.processes_native_event(WindowEvent::MouseInput {
          device_id,
          state: if i % 2 == 0 {
            ElementState::Pressed
          } else {
            ElementState::Released
          },
          button: MouseButton::Left,
          modifiers: ModifiersState::default(),
        });
      });

    local_pool.run();
    assert_eq!(*count.borrow(), 0);
  }

  #[test]
  fn tripe_tap() {
    let (mut wnd, count) = env(3);

    let mut local_pool = LocalPool::new();
    let device_id = unsafe { DeviceId::dummy() };
    observable::interval(Duration::from_millis(10), local_pool.spawner())
      .take(12)
      .subscribe(move |i| {
        wnd.processes_native_event(WindowEvent::MouseInput {
          device_id,
          state: if i % 2 == 0 {
            ElementState::Pressed
          } else {
            ElementState::Released
          },
          button: MouseButton::Left,
          modifiers: ModifiersState::default(),
        });
      });

    local_pool.run();

    assert_eq!(*count.borrow(), 2);
  }
}
