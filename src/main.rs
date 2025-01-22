use ecolor::Color32;
use eframe::egui::{
    self, load::TexturePoll, ColorImage, Event, ImageData, TextureHandle, TextureOptions, Vec2,
};
use egui_plot::{Legend, Plot, PlotImage, PlotPoint};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([350.0, 200.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App with a plot",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MyApp>::default())
        }),
    )
}

fn simulate_robot(display_texture: Arc<Mutex<TextureHandle>>) {
    loop {
        thread::sleep(Duration::from_millis(1));
    }
}

#[derive(Default)]
struct MyApp {
    screen_texture: Option<Arc<Mutex<TextureHandle>>>,
}

impl MyApp {
    #[allow(dead_code)]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let screen_texture = Arc::new(Mutex::new(cc.egui_ctx.load_texture(
            "screen",
            ImageData::Color(Arc::new(ColorImage::new([100, 100], Color32::TRANSPARENT))),
            TextureOptions::default(),
        )));

        print!("Hello World");

        let thread_texture = screen_texture.clone();

        thread::spawn(|| {
            simulate_robot(thread_texture);
        });

        Self {
            screen_texture: Some(screen_texture),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Get scroll input
            let scroll = ui.input(|i| {
                i.events.iter().find_map(|e| match e {
                    Event::MouseWheel {
                        unit: _,
                        delta,
                        modifiers: _,
                    } => Some(*delta),
                    _ => None,
                })
            });

            let my_plot = Plot::new("My Plot").legend(Legend::default());

            my_plot.allow_scroll(false).show(ui, |plot_ui| {
                // Support scroll to zoom
                if let Some(mut scroll) = scroll {
                    scroll = Vec2::splat(scroll.x + scroll.y);
                    let zoom_factor =
                        Vec2::from([(scroll.x / 10.0).exp(), (scroll.y / 10.0).exp()]);

                    plot_ui.zoom_bounds_around_hovered(zoom_factor);
                }

                let texture_poll = ctx
                    .try_load_texture(
                        "file://./src/map.png",
                        egui::TextureOptions::default(),
                        egui::SizeHint::default(),
                    )
                    .unwrap();

                if let TexturePoll::Ready { texture } = texture_poll {
                    plot_ui.image(
                        PlotImage::new(
                            texture.id,
                            PlotPoint { x: 0.0, y: 0.0 },
                            Vec2 { x: 100.0, y: 100.0 },
                        )
                        .tint(Color32::DARK_GRAY),
                    );
                }
            });
        });
    }
}
