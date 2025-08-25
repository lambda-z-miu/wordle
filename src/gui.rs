use crate::common;

use eframe::egui;
use egui::StrokeKind::Inside;
use common::MyApp;


const ui_green : egui::Color32 = egui::Color32::from_rgb(107,186,107);
const ui_yellow : egui::Color32 = egui::Color32::from_rgb(243,194,55);
const ui_red : egui::Color32 = egui::Color32::from_rgb(207,86,125);
const ui_grey : egui::Color32 = egui::Color32::from_rgb(128,128,128);



impl MyApp {
    fn add_char(&mut self, new_letter : char ){
        for rowindex in 0..6{
            for colindex in 0..5{
                if self.board_letter[rowindex][colindex]==None{
                    self.board_letter[rowindex][colindex] = Some(new_letter);
                    return;
                }
            }
        }
    }
}




impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut fonts = egui::FontDefinitions::default();
        // 加载外部字体
        fonts.font_data.insert(
            "ARIAL_BOLD".to_owned(),
            egui::FontData::from_static(include_bytes!("../font.TTF")).into(),
        );

        // 在 Proportional 字体族里插入
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "ARIAL_BOLD".to_owned());

        ctx.set_fonts(fonts);


        for i in self.board_letter{
            for j in i{
                if j != None{
                    // println!("{}",j.expect("UR"));
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {

            /* 
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Wordle").size(48.0) .strong());
                ui.add_space(10.0);
            }); */

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
            

            // === 棋盘 (6 行 5 列) ===
            ui.separator();
            ui.add_space(30.0);
            
            // 一些参数
            const COLS: usize = 5;
            const TILE: f32 = 60.0;   // 方块边长
            const GAP: f32 = 5.0;     // 列间距
            const ROW_GAP: f32 = 5.0; // 行间距


            for row in 0..6 {
                // 计算这一行的总宽度（所有方块 + 列间距）
                let total_row_w = COLS as f32 * TILE + (COLS.saturating_sub(1)) as f32 * GAP;
                // 当前可用宽度
                let avail = ui.available_width();
                // 让这一行整体居中所需的左侧留白
                let left_pad = ((avail - total_row_w) * 0.5).max(0.0);

                // 这一行用水平布局：先塞左侧留白，再依次画 5 个方块
                ui.horizontal(|ui| {
                    // 给这一行的左侧留白，从而让整行居中
                        ui.add_space(left_pad);

                    for col in 0..COLS {
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(TILE, TILE),
                            egui::Sense::click(),
                        );
                        let painter = ui.painter_at(rect);

                        // 背景色: 已点击绿色，否则灰色


                        let fill_color = {
                            match &self.board_color[row][col] {
                                Some(color) => match &color {
                                    common::Color::RED => ui_red,
                                    common::Color::YELLOW => ui_yellow,
                                    common::Color::GREEN => ui_green,
                                    common::Color::GREY => ui_grey,
                                },
                                None => egui::Color32::WHITE,
                        }
                        };

                        let stroke_color = egui::Color32::GRAY;       // 边框灰色
                        let stroke = egui::Stroke::new(2.0, stroke_color); // 边框粗细和颜色

                        // 先画填充
                                painter.rect_filled(rect, 5.0, fill_color);
                                painter.rect_stroke(rect, 5.0, stroke,Inside);
                            
                        let ch = self.board_letter[row][col];
                        if let Some(letter) = ch{
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                letter.to_string(),
                                egui::FontId::proportional(35.0), // 字号
                                egui::Color32::WHITE,             // 白色字
                            );
                        }
                        
                        // 点击切换状态
                        if response.clicked() {
                            self.board[row][col] = !self.board[row][col];
                        }

                        // 列间距（最后一列不用加）
                        if col + 1 != COLS {
                            ui.add_space(GAP);
                        }
                    }
                });

                // 行间距（最后一行可选不加）    
                ui.add_space(ROW_GAP);
            }

            ui.add_space(30.0);

            ui.separator();

            // === 虚拟键盘 A-Z ===
            ui.label("虚拟键盘:");
            for chunk in ('A'..='Z').collect::<Vec<_>>().chunks(9) {
                ui.horizontal(|ui| {
                    for &ch in chunk {
                        if ui.button(ch.to_string()).clicked() {
                            self.selected_key = Some(ch);
                        }
                    }
                });
            }

            if let Some(ch) = self.selected_key {
                ui.label(format!("最近点击的键: {}", ch));
            }

            ui.separator();

            // === 下拉菜单（难度选择） ===
            egui::ComboBox::from_label("Select Difficulty")
                .selected_text(&self.difficulty)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.difficulty, "Easy".to_string(), "Easy");
                    ui.selectable_value(&mut self.difficulty, "Difficult".to_string(), "Difficult");
                });
        });
    }
}





