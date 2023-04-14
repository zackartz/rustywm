use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_data_device, delegate_seat, delegate_shm,
    desktop::{Space, Window},
    input::{pointer::CursorImageStatus, SeatHandler, SeatState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Clock, Logical, Monotonic, Point, Serial},
    wayland::{
        buffer::BufferHandler,
        compositor::{with_states, CompositorHandler, CompositorState},
        data_device::{
            ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
        },
        output::OutputManagerState,
        shell::xdg::{XdgShellHandler, XdgShellState, XdgToplevelSurfaceData},
        shm::{ShmHandler, ShmState},
    }, delegate_xdg_shell, delegate_output,
};

pub struct State {
    pub clock: Clock<Monotonic>,
    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub seat_state: SeatState<Self>,
    pub shm_state: ShmState,
    pub space: Space<Window>,
    pub cursor_status: CursorImageStatus,
    pub pointer_location: Point<f64, Logical>,
    pub output_manager_state: OutputManagerState,
    pub xdg_shell_state: XdgShellState,
}

impl State {
    pub fn refresh_geometry(&mut self) {
        let space = &mut self.space;

        let output = space.outputs().next().cloned().unwrap();

        let output_geometry = space.output_geometry(&output).unwrap();
        let output_width = output_geometry.size.w;
        let output_height = output_geometry.size.h;

        let gap = 6;
        let elements_count = space.elements().count() as i32;

        let mut resizes = vec![];

        for (i, window) in space.elements().enumerate() {
            let (mut x, mut y) = (gap, gap);

            let (mut width, mut height) = (output_width - gap * 2, output_height - gap * 2);

            if elements_count > 1 {
                width -= gap;
                width /= 2;
            }

            if i > 0 {
                height /= elements_count - 1;

                x += width + gap;
                y += height * (i as i32 - 1);
            }

            if i > 1 {
                height -= gap;
                y += gap;
            }

            resizes.push((window.clone(), (width, height), (x, y)));
        }

        for (window, dimensions, position) in resizes {
            window.toplevel().with_pending_state(|state| {
                state.size = Some(dimensions.into());
            });

            window.toplevel().send_configure();

            space.map_element(window, position, false);
        }
    }
}

impl BufferHandler for State {
    fn buffer_destroyed(
        &mut self,
        buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
    }
}

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn commit(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) {
        on_commit_buffer_handler(surface);

        if let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()
        {
            window.on_commit();

            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            });

            if !initial_configure_sent {
                window.toplevel().send_configure();
            }
        }
    }
}
delegate_compositor!(State);

impl ClientDndGrabHandler for State {}
impl ServerDndGrabHandler for State {}

impl DataDeviceHandler for State {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}
delegate_data_device!(State);

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(&mut self, _seat: &smithay::input::Seat<Self>, _image: CursorImageStatus) {
        self.cursor_status = _image;
    }

    fn focus_changed(
        &mut self,
        _seat: &smithay::input::Seat<Self>,
        _focused: Option<&Self::KeyboardFocus>,
    ) {
    }
}
delegate_seat!(State);

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}
delegate_shm!(State);

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let window = Window::new(surface);
        self.space.map_element(window, (0, 0), false);

        self.refresh_geometry();
    }

    fn new_popup(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, positioner: smithay::wayland::shell::xdg::PositionerState) {
        
    }

    fn move_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface, seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat, serial: Serial) {
        
    }

    fn resize_request(
            &mut self,
            surface: smithay::wayland::shell::xdg::ToplevelSurface,
            seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
            serial: Serial,
            edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
        ) {
        
    }

    fn grab(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat, serial: Serial) {
        
    }
}

delegate_xdg_shell!(State);

delegate_output!(State);
