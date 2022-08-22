#![allow(clippy::clone_on_copy)]

use std::{env, fs, io::Read, process::exit};

use drag_controller::{Drag, DragController};
use piston_window::{
    clear, line, math::Matrix2d, Event, Input, Motion, PistonWindow, Transformed, WindowSettings,
};
use svg2polylines::{self, Polyline};

fn main() {
    // Logging
    env_logger::init();

    // Argument parsing
    let args: Vec<_> = env::args().collect();
    match args.len() {
        2 => {}
        _ => {
            println!("Usage: {} <path/to/file.svg>", args[0]);
            exit(1);
        }
    };

    // Load file
    let mut file = fs::File::open(&args[1]).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    // Parse data
    let polylines: Vec<Polyline> = svg2polylines::parse(&s, 0.15, true).unwrap_or_else(|e| {
        println!("Error: {}", e);
        exit(2);
    });
    if polylines.is_empty() {
        println!("Error: No polylines found in input file.");
        exit(2);
    }

    // Create window
    let scale = 2; // Window scaling
    let mut zoom = 1.0;
    let window_size = [716 * scale, 214 * scale];
    let mut window: PistonWindow = WindowSettings::new("Preview (press ESC to exit)", window_size)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Show window
    let black = [0.0, 0.0, 0.0, 1.0];
    let radius = 1.0;
    let mut drag = DragController::new();
    let mut translate: Matrix2d = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let mut translate_tmp: Matrix2d = translate.clone();
    let mut translate_start = None;
    while let Some(e) = window.next() {
        // Handle mouse wheel events
        if let Event::Input(Input::Move(Motion::MouseScroll([_, y])), _) = e {
            if y > 0.0 {
                // Zoom in by 10%
                zoom *= 1.1;
            } else if y < 0.0 {
                // Zoom out by 10%
                zoom *= 0.9;
            }
        };

        // Handle dragging
        drag.event(&e, |action| {
            match action {
                Drag::Start(x, y) => {
                    translate_start = Some((x, y));
                    true
                }
                Drag::Move(x, y) => {
                    let start_x = translate_start.unwrap().0;
                    let start_y = translate_start.unwrap().1;
                    translate_tmp = translate.trans(x - start_x, y - start_y);
                    true
                }
                Drag::End(..) => {
                    translate_start = None;
                    translate = translate_tmp;
                    false
                }
                // Continue dragging when receiving focus.
                Drag::Interrupt => true,
            }
        });

        // Redraw
        window.draw_2d(&e, |ctx, g, _device| {
            clear([1.0; 4], g);
            for polyline in &polylines {
                for pair in polyline.as_ref().windows(2) {
                    line(
                        black,
                        radius,
                        [pair[0].x, pair[0].y, pair[1].x, pair[1].y],
                        ctx.transform
                            .append_transform(translate_tmp)
                            .scale(zoom, zoom),
                        g,
                    );
                }
            }
        });
    }
}
