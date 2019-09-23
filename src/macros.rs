#[macro_export]
macro_rules! replace {
  (($($_:tt)*); $($t:tt)*) => {
    $($t)*
  };
}

#[macro_export]
macro_rules! define {
  ($($vis:vis fn $func:ident($($p:ident),*) -> ($($o:ident),*) {$($tt:tt)*})*) => {$(
    #[must_use]
    $vis fn $func($($p: usize),*) -> impl FnOnce(&mut $crate::circuit::Circuit $(,$crate::replace!(($o); usize))*) {
      move |circuit, $($o),*| {
        $crate::circuit!(circuit; $($tt)*);
      }
    }
  )*};
}

#[macro_export]
macro_rules! circuit {
  ($self:expr; $($func:ident)::+($($p:tt)*); $($tt:tt)*) => {
    $($func)::+($($p)*)($self);
    $crate::circuit!($self; $($tt)*);
  };
  ($self:expr; $o:ident = $($func:ident)::+($($p:tt)*); $($tt:tt)*) => {
    $crate::circuit!($self; ($o) = $($func)::+($($p)*););
    $crate::circuit!($self; $($tt)*);
  };
  ($self:expr; let $o:ident = $($func:ident)::+($($p:tt)*); $($tt:tt)*) => {
    $crate::circuit!($self; let ($o) = $($func)::+($($p)*););
    $crate::circuit!($self; $($tt)*);
  };
  ($self:expr; ($($o:ident),*) = $($func:ident)::+($($p:tt)*); $($tt:tt)*) => {
    $($func)::+($($p)*)($self $(,$o)*);
    $crate::circuit!($self; $($tt)*);
  };
  ($self:expr; let ($($o:ident),*) = $($func:ident)::+($($p:tt)*); $($tt:tt)*) => {
    let f = $($func)::+($($p)*);
    $(let $o = $self.add_wire();)*
    f($self $(,$o)*);
    $crate::circuit!($self; $($tt)*);
  };
  ($self:expr;) => {};
}
