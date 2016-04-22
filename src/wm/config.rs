use wm::client::{TagSet, TagStack};
use wm::kbd::*;

use wm::layout::monocle::Monocle;
use wm::layout::vstack::VStack;
use wm::layout::hstack::HStack;
use wm::layout::dstack::DStack;

use wm::window_system::{Wm,WmConfig};

// a set of (symbolic) tags - to be extended/modified
#[derive(Debug, PartialEq, Clone)]
pub enum Tag {
    Foo,
    Bar,
    Baz
}

impl Tag {
    pub fn default() -> Tag {
        Tag::Foo
    }
}

// generate a window manager config - colors, border width...
pub fn generate_config() -> WmConfig {
    WmConfig {
        f_color: (0xffff, 0x0, 0x0),
        u_color: (0x00, 0x00, 0x00),
        border_width: 1,
    }
}

// setup datastructures for the window manager, ie keybindings and tagstack
pub fn setup_wm(wm: &mut Wm) {
    wm.setup_bindings(
        vec![(KeyPress{code: 42, mods: 8}, Box::new(|_, s| s.swap_top())),
             (KeyPress{code: 10, mods: 8}, Box::new(|_, s| s.swap_nth(0))),
             (KeyPress{code: 11, mods: 8}, Box::new(|_, s| s.swap_nth(1))),
             (KeyPress{code: 12, mods: 8}, Box::new(|_, s| s.swap_nth(2))),
             (KeyPress{code: 13, mods: 8}, Box::new(|_, s| s.swap_nth(3))),
             (KeyPress{code: 14, mods: 8}, Box::new(|_, s| s.swap_nth(4))),
             (KeyPress{code: 15, mods: 8}, Box::new(|_, s| s.swap_nth(5))),
             (KeyPress{code: 16, mods: 8}, Box::new(|_, s| s.swap_nth(6))),
             (KeyPress{code: 17, mods: 8}, Box::new(|_, s| s.swap_nth(7))),
             (KeyPress{code: 18, mods: 8}, Box::new(|_, s| s.swap_nth(8))),
             (KeyPress{code: 19, mods: 8}, Box::new(|_, s| s.swap_nth(9))),
        ]
    );
    wm.setup_tags(TagStack::from_vec(
        vec![TagSet::new(vec![Tag::Foo], Monocle::default()),
             TagSet::new(vec![Tag::Baz], DStack::default()),
             TagSet::new(vec![Tag::Foo], VStack::default()),
             TagSet::new(vec![Tag::Bar], HStack::default())
        ]
    ));
}