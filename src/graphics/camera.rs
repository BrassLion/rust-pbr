use nalgebra::*;

pub struct Camera {
    pub view_matrix: Isometry3<f32>,
    pub proj_matrix: Perspective3<f32>,
    camera_up: Vector3<f32>,
    camera_target: Vector3<f32>,

    current_button_pressed: Option<winit::event::MouseButton>,
    is_first_mouse_press: bool,
    last_mouse_pos: Vector2<f32>,

    trans_scaling_factor: f32,
    rot_scaling_factor: f32,
    zoom_scaling_factor: f32,
    zoom_min_distance: f32,
    zoom_max_distance: f32,
}

impl Camera {
    pub fn new(
        eye: &Point3<f32>,
        target: &Point3<f32>,
        up: &Vector3<f32>,
        aspect_ratio: f32,
        fov_y: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let view_matrix = Isometry3::look_at_rh(eye, target, up);
        let proj_matrix = Perspective3::new(aspect_ratio, fov_y, z_near, z_far);

        Self {
            view_matrix,
            proj_matrix,
            camera_up: *up,
            camera_target: Vector3::new(target.x, target.y, target.z),
            current_button_pressed: None,
            is_first_mouse_press: false,
            last_mouse_pos: Vector2::zeros(),
            trans_scaling_factor: 0.01,
            rot_scaling_factor: 2.0,
            zoom_scaling_factor: 0.05,
            zoom_min_distance: 1.0,
            zoom_max_distance: 50.0,
        }
    }

    fn update_camera_zoom(&mut self, zoom_magnitude: f32) {
        let mut transform = self.view_matrix.inverse();

        let direction = Unit::new_normalize(transform.translation.vector - self.camera_target);
        let direction_scaled = direction.scale(zoom_magnitude * self.zoom_scaling_factor);
        let mut new_position = transform.translation.vector + direction_scaled;

        // Clamp to min zoom distance.
        if new_position.dot(&direction) < self.zoom_min_distance {
            new_position = self.camera_target + direction.scale(self.zoom_min_distance);
        }
        // Clamp to max zoom distance.
        else if new_position.dot(&direction) > self.zoom_max_distance {
            new_position = self.camera_target + direction.scale(self.zoom_max_distance);
        }

        transform.translation.vector = new_position;
        self.view_matrix = transform.inverse();
    }

    fn update_camera_rotation(&mut self, window_size: [f32; 2], mouse_position: &Vector2<f32>) {
        let get_mouse_pos_on_arcball = |x, y| {
            let mut point_on_ball = Vector3::new(
                x as f32 / window_size[0] as f32 * 2.0 - 1.0,
                1.0 - y as f32 / window_size[1] as f32 * 2.0,
                0.0,
            );

            let xy_squared = point_on_ball.x * point_on_ball.x + point_on_ball.y * point_on_ball.y;

            if xy_squared <= 1.0 {
                point_on_ball.z = (1.0 - xy_squared).sqrt();
            } else {
                point_on_ball.normalize_mut();
            }

            point_on_ball
        };

        // Calculate rotation in camera space.
        let last_pos = get_mouse_pos_on_arcball(mouse_position.x as f32, mouse_position.y as f32);
        let cur_pos = get_mouse_pos_on_arcball(self.last_mouse_pos.x, self.last_mouse_pos.y);

        let angle = last_pos.dot(&cur_pos).min(1.0).acos() * self.rot_scaling_factor;

        let axis_in_camera = Unit::new_normalize(last_pos.cross(&cur_pos));

        // Calculate rotation in world space.
        let mut transform = self.view_matrix.inverse();

        let axis_in_world = transform * axis_in_camera;

        let rotation_in_world = UnitQuaternion::from_axis_angle(&axis_in_world, angle);

        // Apply rotation around target.
        transform.append_rotation_wrt_point_mut(&rotation_in_world, &self.camera_target.into());

        self.view_matrix = transform.inverse();
    }

    fn update_camera_position(&mut self, mouse_position: &Vector2<f32>) {
        let mut transform = self.view_matrix.inverse();

        let world_camera_up = (transform * self.camera_up).normalize();
        let world_camera_right = world_camera_up
            .cross(&(transform.translation.vector - self.camera_target))
            .normalize();

        let mouse_delta = (mouse_position - self.last_mouse_pos).scale(self.trans_scaling_factor);

        self.camera_target +=
            world_camera_up.scale(mouse_delta.y) + world_camera_right.scale(mouse_delta.x);
        transform.translation.vector +=
            world_camera_up.scale(mouse_delta.y) + world_camera_right.scale(mouse_delta.x);

        self.view_matrix = transform.inverse();
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::MouseInput { button, state, .. } => {
                if *state == winit::event::ElementState::Pressed {
                    self.current_button_pressed = Some(*button);
                    self.is_first_mouse_press = true;
                } else {
                    self.current_button_pressed = None;
                }
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let current_mouse_pos = Vector2::new(position.x as f32, position.y as f32);

                if self.is_first_mouse_press {
                    self.last_mouse_pos = current_mouse_pos;
                    self.is_first_mouse_press = false;
                    return;
                }

                match self.current_button_pressed {
                    Some(winit::event::MouseButton::Left) => {
                        self.update_camera_rotation(window.inner_size().into(), &current_mouse_pos);
                    }
                    Some(winit::event::MouseButton::Right) => {
                        self.update_camera_position(&current_mouse_pos);
                    }
                    _ => {}
                }

                self.last_mouse_pos = current_mouse_pos;
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::PixelDelta(winit::dpi::LogicalPosition {
                    y,
                    ..
                }) => {
                    self.update_camera_zoom(*y as f32);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
