#[derive(PartialEq)]
pub enum Color{
    RED,
    YELLOW,
    GREEN,
    GREY,
}

#[derive(Default)]
pub struct MyApp {
    pub board: [[bool; 5]; 6],     // 5列x6行, true = 已点击

    pub board_letter : [[Option<char> ; 5 ] ; 6 ],
    pub board_color  : [[Option<Color> ; 5 ] ; 6 ],

    pub selected_key: Option<char>, // 最近点击的字母
    pub difficulty: String,         // 下拉菜单选择

}