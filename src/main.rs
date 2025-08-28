mod common;
mod gamelogic;
mod parseconfig;

use common::MyApp;
use eframe::egui;
use egui::RichText;
use egui::{Color32, StrokeKind::Inside};
use egui::{Pos2, Vec2};

const UI_GREEN: egui::Color32 = egui::Color32::from_rgb(107, 186, 107);
const UI_YELLOW: egui::Color32 = egui::Color32::from_rgb(243, 194, 55);
const UI_RED: egui::Color32 = egui::Color32::from_rgb(207, 86, 125);
const UI_GREY: egui::Color32 = egui::Color32::from_rgb(128, 128, 128);

impl MyApp {
    /// add char to Myapp::board_letter
    fn add_char(&mut self, new_letter: char) {
        for rowindex in 0..6 {
            if self.board_letter[rowindex][4].is_some() && !self.row_lock[rowindex] {
                return; // when there is a line is full and not checked, add cannot be called 
            }

            if self.board_letter[rowindex][4].is_some() {
                continue; // when a line is full, it can not be added but others might
            }

            for colindex in 0..5 {
                if self.board_letter[rowindex][colindex] == None {
                    self.board_letter[rowindex][colindex] = Some(new_letter);
                    // println!("added char {} at {},{}",new_letter,rowindex+1,colindex+1);
                    return;
                }
            }
        }
    }

    /// delete char to Myapp::board_letter
    fn del_char(&mut self) {
        for rowindex in 0..6 {
            if self.row_lock[rowindex] || self.board_letter[rowindex][0].is_none() {
                // println!("row {} locked or empt",rowindex);
                continue; // when a line is empty or is locked, it cannot be deleted but otherlines might;
            }

            for colindex in (0..5).rev() {
                if self.board_letter[rowindex][colindex].is_some() {
                    // println!("deleted char {} at {},{}",self.board_letter[rowindex][colindex].unwrap(),rowindex+1,colindex+1);
                    self.board_letter[rowindex][colindex] = None;
                    return;
                }
            }
        }
    }

    fn spawn_confetti(&mut self, ctx: &egui::Context) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let screen = ctx.screen_rect();
        let center_x = screen.center().x;
        let top_y = screen.top();

        for _ in 0..8 {
            self.confetti.push(common::Confetti {
                pos: Pos2::new(center_x + rng.gen_range(-800.0..800.0), top_y),
                vel: Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(2.0..5.0)),
                color: Color32::from_rgb(
                    rng.gen_range(0..256) as u8,
                    rng.gen_range(0..256) as u8,
                    rng.gen_range(0..256) as u8,
                ),
                lifetime: 6.0,
            });
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "ARIAL_BOLD".to_owned(),
            egui::FontData::from_static(include_bytes!("../font.TTF")).into(),
        );

        fonts.font_data.insert(
            "SYMBOLS".to_owned(),
            egui::FontData::from_static(include_bytes!("../SEGMDL2.TTF")).into(),
        );

        // add font family
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "ARIAL_BOLD".to_owned());

        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "SYMBOLS".to_owned());

        ctx.set_fonts(fonts);

        for i in self.board_letter {
            for j in i {
                if j != None {
                    // println!("{}",j.expect("UR"));
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let dt = 1.0 / 30.0; // 30fps
                self.time += dt;

                for c in &mut self.confetti {
                    c.pos += c.vel * dt * 60.0; 
                    c.lifetime -= dt;
                }

                // delete invalid confetti
                self.confetti.retain(|c| c.lifetime > 0.0);
                let painter = ui.painter();
                for c in &self.confetti {
                    painter.circle_filled(c.pos, 5.0, c.color);
                    ctx.request_repaint();
                }
            });


            ui.input(|i| {
                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        if let Some(ch) = text.chars().next() {
                            if ch.is_ascii_alphabetic() {
                                self.add_char(ch.to_ascii_uppercase());
                            }
                        }
                    }
                }
            });

            ui.input(|i| {
                if i.key_pressed(egui::Key::Backspace) {
                    self.del_char();
                }

                if i.key_pressed(egui::Key::Enter) {
                    self.entered = true;
                    // print!("enter+pressed")
                }
            });

            for j in 0..6 {
                if self.row_lock[j] {
                    continue;
                }
                let mut guess: String = String::new();

                if !self.board_letter[j][4].is_some() && self.entered {
                    self.entered = false;
                }
                if self.board_letter[j][4].is_some() && self.entered {
                    for i in 0..5 {
                        guess.push(self.board_letter[j][i].expect("UNREACHABLE"));
                    }

                    let checker = match self.config.difficult {
                        true => gamelogic::check_valid_guess_difficult,
                        false => gamelogic::check_valid_guess,
                    };

                    if !checker(guess.clone(), &self.game_state) {
                        for i in 0..5 {
                            self.board_letter[j][i] = None;
                            self.board_color[j][i] = None;
                        }
                    } else {
                        let round_result = gamelogic::game_round(
                            &mut self.game_state,
                            guess.clone(),
                        );
                        self.winflag = true;
                        for i in round_result.1 {
                            if i != common::Color::GREEN {
                                self.winflag = false;
                            }
                        }
                        for i in 0..5 {
                            self.board_color[j][i] = Some(round_result.1[i]);
                        }
                        self.row_lock[j] = true;
                        gamelogic::paint_keyboad(&mut self.game_state, round_result.1, &guess);
                    }
                    self.entered = false;
                }
            }

            // board
            ui.separator();
            ui.add_space(30.0);

            // UI params
            const COLS: usize = 5;
            const TILE: f32 = 60.0;
            const GAP: f32 = 5.0;
            const ROW_GAP: f32 = 5.0;

            for row in 0..6 {
                let total_row_w = COLS as f32 * TILE + (COLS.saturating_sub(1)) as f32 * GAP;
                let avail = ui.available_width();
                let left_pad = ((avail - total_row_w) * 0.5).max(0.0);

                ui.horizontal(|ui| {
                    ui.add_space(left_pad);

                    for col in 0..COLS {
                        let (rect, _response) =
                            ui.allocate_exact_size(egui::vec2(TILE, TILE), egui::Sense::click());
                        let painter = ui.painter_at(rect);


                        let fill_color = {
                            match &self.board_color[row][col] {
                                Some(color) => match &color {
                                    common::Color::RED => UI_RED,
                                    common::Color::YELLOW => UI_YELLOW,
                                    common::Color::GREEN => UI_GREEN,
                                    common::Color::GREY => UI_GREY,
                                },
                                None => egui::Color32::WHITE,
                            }
                        };

                        let stroke_color = egui::Color32::GRAY; // grey edge
                        let stroke = egui::Stroke::new(2.0, stroke_color); 

                        painter.rect_filled(rect, 5.0, fill_color);
                        painter.rect_stroke(rect, 5.0, stroke, Inside);

                        let ch = self.board_letter[row][col];
                        if let Some(letter) = ch {
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                letter.to_string(),
                                egui::FontId::proportional(35.0), 
                                {
                                    match self.board_color[row][col] {
                                        None => egui::Color32::BLACK,
                                        Some(_) => egui::Color32::WHITE,
                                    }
                                },
                            );
                        }

                        if col + 1 != COLS {
                            ui.add_space(GAP);
                        }
                    }
                });

                ui.add_space(ROW_GAP);
            }

            ui.add_space(30.0);
            if self.row_lock[5] && !self.winflag {
                ui.vertical_centered(|ui| {
                    let ret: String = "Failed! The answer is ".to_string() + &self.game_state.word;
                    let restart = ui.button(egui::RichText::new(&ret).size(30.0).strong());
                    if restart.clicked() {
                        self.winflag = false;
                        gamelogic::reset_game_state(&self.config, &mut self.game_state);
                        self.board_letter = [[None; 5]; 6];
                        self.board_color = [[None; 5]; 6];
                        self.row_lock = [false; 6];
                    }
                    ui.add_space(30.0);
                });
            }
            if self.winflag {
                ui.vertical_centered(|ui| {
                    let restart =
                        ui.button(egui::RichText::new("🎉 You Win! 🎉").size(30.0).strong());
                    if restart.clicked() {
                        self.winflag = false;
                        gamelogic::reset_game_state(&self.config, &mut self.game_state);
                        self.board_letter = [[None; 5]; 6];
                        self.board_color = [[None; 5]; 6];
                        self.row_lock = [false; 6];
                    }
                    ui.add_space(30.0);
                });
                self.spawn_confetti(ctx);
            } else {
                ui.add_space(60.0);
            }

            // Virtual keyboard

            const ROW1_LEN: f32 = 54.0;
            const ROW1_HEIGHT: f32 = 50.0;
            const ROW1_GAP: f32 = 5.0;
            const ROW2_LEN: f32 = 60.0;
            const ROW2_GAP: f32 = 6.5;
            const LONG_KEY: f32 = 87.0;

            let avail = ui.available_width();
            let total_row1 = 10 as f32 * ROW1_LEN + 9 as f32 * ROW1_GAP;
            let left_pad_1 = (avail - total_row1) * 0.5;

            ui.horizontal(|ui| {
                ui.add_space(left_pad_1);

                for letter in (['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'])
                    .into_iter()
                    .collect::<Vec<_>>()
                {
                    let bg_color = self.game_state.alphabet[&letter];
                    let btn_color = match bg_color {
                        common::Color::RED => UI_RED,
                        common::Color::YELLOW => UI_YELLOW,
                        common::Color::GREEN => UI_GREEN,
                        common::Color::GREY => UI_GREY,
                    };
                    let btn = ui.add_sized(
                        [ROW1_LEN, ROW1_HEIGHT],
                        egui::Button::new(
                            RichText::new(letter.to_string())
                                .size(20.0)
                                .color(Color32::WHITE),
                        )
                        .fill(btn_color),
                    ); //.color(Color32::BLACK)
                    ui.add_space(ROW1_GAP);

                    if btn.clicked() {
                        self.add_char(letter);
                    }
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.add_space(left_pad_1);
                for letter in (['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'])
                    .into_iter()
                    .collect::<Vec<_>>()
                {
                    let bg_color = self.game_state.alphabet[&letter];
                    let btn_color = match bg_color {
                        common::Color::RED => UI_RED,
                        common::Color::YELLOW => UI_YELLOW,
                        common::Color::GREEN => UI_GREEN,
                        common::Color::GREY => UI_GREY,
                    };
                    let btn = ui.add_sized(
                        [ROW2_LEN, ROW1_HEIGHT],
                        egui::Button::new(
                            RichText::new(letter.to_string())
                                .size(20.0)
                                .color(Color32::WHITE),
                        )
                        .fill(btn_color),
                    );
                    ui.add_space(ROW2_GAP);

                    if btn.clicked() {
                        self.add_char(letter);
                    }
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.add_space(left_pad_1);

                let btn_bk = ui.add_sized(
                    [LONG_KEY, ROW1_HEIGHT],
                    egui::Button::new(RichText::new("\u{E750}").size(20.0)),
                );
                if btn_bk.clicked() {
                    self.del_char();
                }
                ui.add_space(ROW1_GAP);

                for letter in (['Z', 'X', 'C', 'V', 'B', 'N', 'M'])
                    .into_iter()
                    .collect::<Vec<_>>()
                {
                    let bg_color = self.game_state.alphabet[&letter];
                    let btn_color = match bg_color {
                        common::Color::RED => UI_RED,
                        common::Color::YELLOW => UI_YELLOW,
                        common::Color::GREEN => UI_GREEN,
                        common::Color::GREY => UI_GREY,
                    };
                    let btn = ui.add_sized(
                        [ROW1_LEN, ROW1_HEIGHT],
                        egui::Button::new(
                            RichText::new(letter.to_string())
                                .size(20.0)
                                .color(Color32::WHITE),
                        )
                        .fill(btn_color),
                    );
                    ui.add_space(ROW1_GAP);
                    if btn.clicked() {
                        self.add_char(letter);
                    }
                }

                let btn_et = ui.add_sized(
                    [LONG_KEY, ROW1_HEIGHT],
                    egui::Button::new(RichText::new("\u{E751}").size(20.0)),
                );
                if btn_et.clicked() {
                    self.entered = true;
                }
            });

            ui.add_space(10.0);
            ui.add_space(30.0);

        });
    }
}

fn main() -> () {
    let final_config = parseconfig::parse_config();
    let common_gs = gamelogic::generate_game_state(&final_config);

    if !final_config.ui {
        gamelogic::pure_game(final_config, common_gs);
        return;
    }

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Wordle UI Demo",
        options,
        Box::new(|_cc| {
            let app = MyApp {
                game_state: common_gs,
                config: final_config,
                ..Default::default()
            };

            Ok(Box::new(app))
        }),
    )
    .expect("a");
}
