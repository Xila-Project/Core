use std::sync::{Arc, Mutex};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    platform::{pump_events::EventLoopExtPumpEvents, wayland::EventLoopBuilderExtWayland},
    window::{Window, WindowId},
};
use File_system::{Create_device, Device_trait, Device_type, Size_type};
use Graphics::{
    Color_RGBA8888_type, Input_data_type, Point_type, Screen_read_data_type,
    Screen_write_data_type, State_type,
};

#[derive(Default)]
struct Window_type {
    Resolution: Point_type,
    Window: Option<Window>,
    Pixels: Option<Pixels>,
    Pointer_data: Input_data_type,
}

impl Window_type {
    fn New(Resolution: Point_type) -> Self {
        Self {
            Resolution,
            ..Default::default()
        }
    }

    fn Get_pointer_data(&self) -> &Input_data_type {
        &self.Pointer_data
    }

    fn Get_resolution(&self) -> Option<Point_type> {
        self.Window.as_ref().map(|Window| {
            let Size = Window.inner_size();
            Point_type::New(Size.width as i16, Size.height as i16)
        })
    }

    fn Draw(&mut self, Data: &Screen_write_data_type) -> Result<(), String> {
        let Frame_width = self.Resolution.Get_x() as usize;
        let Data_area = Data.Get_area();

        let Point_1 = Data_area.Get_point_1();
        let Point_2 = Data_area.Get_point_2();

        let Pixels = self
            .Pixels
            .as_mut()
            .ok_or_else(|| "Pixels is None.".to_string())?;

        let Frame = Pixels.frame_mut();
        let Frame = unsafe {
            core::slice::from_raw_parts_mut(
                Frame.as_mut_ptr() as *mut Color_RGBA8888_type,
                Frame.len() / size_of::<Color_RGBA8888_type>(),
            )
        };

        let Data_buffer = Data.Get_buffer();

        let Frame_x_start = Point_1.Get_x() as usize;
        let Frame_y_start = Point_1.Get_y() as usize;
        let Width = (Point_2.Get_x() - Point_1.Get_x() + 1) as usize;
        let Height = (Point_2.Get_y() - Point_1.Get_y() + 1) as usize;

        for (y, Data_row) in Data_buffer.chunks(Width).enumerate().take(Height) {
            let Frame_row_start = (Frame_y_start + y) * Frame_width + Frame_x_start;
            let Frame_row_end = Frame_row_start + Width;
            let Frame_row = &mut Frame[Frame_row_start..Frame_row_end];

            Frame_row
                .iter_mut()
                .zip(Data_row.iter())
                .for_each(|(Destination, &Source)| {
                    let Source = Color_RGBA8888_type::From_RGB565(Source);
                    *Destination = Source;
                });
        }

        // - Request a redraw.
        self.Window
            .as_ref()
            .ok_or_else(|| "Window is None.".to_string())?
            .request_redraw();

        Ok(())
    }
}

impl ApplicationHandler for Window_type {
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(Window) = &self.Window {
            Window.request_redraw();
        }
    }

    fn resumed(&mut self, Event_loop: &ActiveEventLoop) {
        let Window = {
            let Size = LogicalSize::new(
                self.Resolution.Get_x() as f64,
                self.Resolution.Get_y() as f64,
            );

            let Window_attributes = Window::default_attributes()
                .with_title("Xila")
                .with_inner_size(Size)
                .with_min_inner_size(Size);

            Event_loop.create_window(Window_attributes).unwrap()
        };

        let Pixels = {
            let Surface_texture = SurfaceTexture::new(
                self.Resolution.Get_x() as u32,
                self.Resolution.Get_y() as u32,
                &Window,
            );

            Pixels::new(
                self.Resolution.Get_x() as u32,
                self.Resolution.Get_y() as u32,
                Surface_texture,
            )
            .unwrap()
        };

        self.Window = Some(Window);
        self.Pixels = Some(Pixels);
    }

    fn window_event(
        &mut self,
        _: &ActiveEventLoop,
        Window_identifier: WindowId,
        Event: WindowEvent,
    ) {
        let Window = if let Some(Window) = &self.Window {
            Window
        } else {
            return;
        };

        if Window_identifier != Window.id() {
            return;
        }

        match Event {
            WindowEvent::RedrawRequested => {
                if let Some(Pixels) = &mut self.Pixels {
                    Pixels.render().unwrap();
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let ElementState::Pressed = event.state {
                    println!("Key pressed: {:?}", event.logical_key);
                    // - Handle keyboard input.
                } else {
                    println!("Key released: {:?}", event.logical_key);
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position: Position,
            } => self
                .Pointer_data
                .Set_point((Position.x as i16, Position.y as i16).into()),
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button: _,
            } => match state {
                ElementState::Pressed => {
                    self.Pointer_data.Set_touch(State_type::Pressed);
                }

                ElementState::Released => {
                    self.Pointer_data.Set_touch(State_type::Released);
                }
            },
            _ => {}
        }
    }
}

struct Inner_type(Window_type, EventLoop<()>);

unsafe impl Sync for Inner_type {}

unsafe impl Send for Inner_type {}

impl Inner_type {
    fn New(Resolution: Point_type) -> Result<Self, String> {
        let mut Event_loop = EventLoop::builder()
            //  .with_wayland()
            .with_any_thread(true)
            .build()
            .map_err(|Error| format!("Error building event loop: {:?}", Error))?;

        let mut Window = Window_type::New(Resolution);

        Event_loop.pump_app_events(None, &mut Window);

        Ok(Self(Window, Event_loop))
    }

    fn Get_resolution(&self) -> Option<Point_type> {
        self.0.Get_resolution()
    }

    fn Draw(&mut self, Data: &Screen_write_data_type) -> Result<(), String> {
        self.0.Draw(Data)
    }

    fn Get_pointer_data(&mut self) -> Result<&Input_data_type, String> {
        self.1.pump_app_events(None, &mut self.0);

        Ok(self.0.Get_pointer_data())
    }
}

pub struct Screen_device_type(Arc<Mutex<Inner_type>>);

unsafe impl Sync for Screen_device_type {}

unsafe impl Send for Screen_device_type {}

impl Screen_device_type {
    fn New(Inner: Arc<Mutex<Inner_type>>) -> Self {
        Self(Inner)
    }
}

impl Device_trait for Screen_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
        let Data: &mut Screen_read_data_type = Buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        let Resolution = self.0.lock()?.Get_resolution().unwrap();

        Data.Set_resolution(Resolution);

        Ok(Size_type::New(size_of::<Self>() as u64))
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        let Data: &Screen_write_data_type = Buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        self.0.lock()?.Draw(Data).unwrap();

        Ok(Size_type::New(size_of::<Self>() as u64))
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Ok(Size_type::New(size_of::<Self>() as u64))
    }

    fn Set_position(
        &self,
        _: &File_system::Position_type,
    ) -> File_system::Result_type<File_system::Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}

pub struct Pointer_device_type(Arc<Mutex<Inner_type>>);

impl Pointer_device_type {
    fn New(Inner: Arc<Mutex<Inner_type>>) -> Self {
        Self(Inner)
    }
}

impl Device_trait for Pointer_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<Size_type> {
        // - Cast the pointer data to the buffer.
        let Data: &mut Input_data_type = Buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        // Copy the pointer data.
        *Data = *self.0.lock()?.Get_pointer_data().unwrap();

        Ok(size_of::<Input_data_type>().into())
    }

    fn Write(&self, _: &[u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(size_of::<Input_data_type>().into())
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}

pub fn New(Resolution: Point_type) -> Result<(Device_type, Device_type), String> {
    let Inner = Arc::new(Mutex::new(Inner_type::New(Resolution)?));

    let Screen_device = Screen_device_type::New(Inner.clone());

    let Pointer_device = Pointer_device_type::New(Inner);

    Ok((
        Create_device!(Screen_device),
        Create_device!(Pointer_device),
    ))
}
