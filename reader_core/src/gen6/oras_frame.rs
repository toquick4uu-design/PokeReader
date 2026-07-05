use super::{
    draw::{
        draw_daycare, draw_dex_nav, draw_header, draw_mirage_spot, draw_pkx, draw_rng, draw_seed_rng, PkxType,
    },
    reader::Gen6Reader,
    rng::Gen6Rng,
};
use crate::{pnp, request_pause, utils::{
    help_menu::HelpMenu,
    menu::{Menu, MenuOption},
    sub_menu::SubMenu,
    ShowView,
}};
use once_cell::unsync::Lazy;
use crate::gen6::draw::draw_auto_advance;
use crate::utils::number_input::NumberInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrasView {
    MainMenu,
    Rng,
    AutoAdvance,
    Daycare1,
    Daycare2,
    Wild,
    DexNav,
    Party,
    MirageSpot,
    SeedRng,
    HelpMenu,
}

struct PersistedState {
    rng: Gen6Rng,
    show_view: ShowView,
    view: OrasView,
    main_menu: Menu<OrasView>,
    party_menu: SubMenu,
    wild_menu: SubMenu,
    help_menu: HelpMenu,
    memory_ready: bool,
    target_advance_selector: NumberInput<'static, u32>,
    target_advance: Option<u32>,
}

const MENU: &[MenuOption<OrasView>] = &[
    MenuOption::new(OrasView::Rng, "RNG"),
    MenuOption::new(OrasView::AutoAdvance, "Auto Advance MT"),
    MenuOption::new(OrasView::Daycare1, "Daycare"),
    MenuOption::new(OrasView::Daycare2, "Daycare 2"),
    MenuOption::new(OrasView::Wild, "Wild"),
    MenuOption::new(OrasView::DexNav, "DexNav"),
    MenuOption::new(OrasView::Party, "Party"),
    MenuOption::new(OrasView::MirageSpot, "Mirage Spot"),
    MenuOption::new(OrasView::SeedRng, "Seed RNG"),
    MenuOption::new(OrasView::HelpMenu, "Help"),
];

unsafe fn get_state() -> &'static mut PersistedState {
    static mut STATE: Lazy<PersistedState> = Lazy::new(|| PersistedState {
        rng: Gen6Rng::default(),
        show_view: ShowView::default(),
        view: OrasView::MainMenu,
        party_menu: SubMenu::new(1, 6),
        wild_menu: SubMenu::new(1, 5),
        help_menu: HelpMenu::default(),
        main_menu: Menu::new(MENU),
        memory_ready: false,
        target_advance_selector: NumberInput::new("MT Target"),
        target_advance: None,
    });
    Lazy::force_mut(&mut STATE)
}

pub fn oras_clear_pause_update_state() {
    // This is safe as long as this is guaranteed to run single threaded.
    // A lock hinders performance too much on a 3ds.
    let state = unsafe { get_state() };
    state.target_advance = None;
}

pub fn run_oras_frame() {
    pnp::set_print_max_len(23);

    let reader = Gen6Reader::oras();

    // This is safe as long as this is guaranteed to run single threaded.
    // A lock hinders performance too much on a 3ds.
    let state = unsafe { get_state() };

    // Check if memory is mapped before attempting to read
    if !state.memory_ready {
        if !reader.is_memory_ready() {
            return;
        }
        state.memory_ready = true;
    }

    state.rng.update(&reader);

    // If there is an advance target, which is equal or larger than the current advance, trigger a pause
    if let Some(target_advance) = state.target_advance {
        if target_advance <= state.rng.mt().advances() {
            request_pause();
        }
    }

    if !state.show_view.check() {
        return;
    }

    let is_locked = state.main_menu.update_lock();
    state.view = state.main_menu.next_view(OrasView::MainMenu, state.view);

    // Don't display header for the AutoAdvance menu (as the controls are different)
    if !matches!(state.view, OrasView::AutoAdvance) {
        draw_header(OrasView::MainMenu, state.view, is_locked);
    }

    match state.view {
        OrasView::Rng => draw_rng(&reader, &state.rng, &state.target_advance),
        OrasView::AutoAdvance => {
            let current_advance =  state.rng.mt().advances();
            // If we just entered the auto advance page (we know it is the case because main view is not locked yet)
            // Set the value on the selector to either the existing target for edition, or the current advance value if no target is set
            if !is_locked{
                state.target_advance_selector.set_value(state.target_advance.unwrap_or(state.rng.mt().advances()) as usize);
            }

            if pnp::is_just_pressed(pnp::Button::Start) {
                // Expensive-ish operation, only do it when requested
                let selected = state.target_advance_selector.value();
                if selected >= current_advance{
                    state.target_advance = Some(state.target_advance_selector.value());
                } else {
                    state.target_advance_selector.set_value(selected as usize);
                }
            }

            if pnp::is_just_pressed(pnp::Button::Select)  {
                state.main_menu.unlock();
                state.view = OrasView::MainMenu;

                state.main_menu.update_view();
                state.main_menu.draw();
                return;
            }

            // Lock main menu to allow for view controls to not exit the submenu
            state.main_menu.lock();

            state.target_advance_selector.update();

            draw_auto_advance(state.rng.mt().advances(), &state.target_advance, &state.target_advance_selector)
        }
        OrasView::Daycare1 => draw_daycare(&reader.daycare1()),
        OrasView::Daycare2 => draw_daycare(&reader.daycare2()),
        OrasView::Wild => {
            let slot = state.wild_menu.update_and_draw(is_locked);
            draw_pkx(&reader.wild_pkm((slot - 1) as u32), PkxType::Wild);
        }
        OrasView::DexNav => draw_dex_nav(&reader, &state.rng, &state.target_advance),
        OrasView::Party => {
            let slot = state.party_menu.update_and_draw(is_locked);
            draw_pkx(&reader.party_pkm((slot - 1) as u32), PkxType::Tame);
        }
        OrasView::SeedRng => draw_seed_rng(&reader, &state.rng, &state.target_advance),
        OrasView::MirageSpot => draw_mirage_spot(&reader),
        OrasView::HelpMenu => state.help_menu.update_and_draw(is_locked),
        OrasView::MainMenu => {
            state.main_menu.update_view();
            state.main_menu.draw();
        },
    }
}
