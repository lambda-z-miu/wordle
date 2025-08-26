# Wordle with GUI 踩坑记录~~反省~~

疑似上次自己写超过1000行的项目还是软设，大二一年没怎么写代码倒是学了不少数学课，sf虽然并没有增长代码能力，但是极大地提高了我纸笔分类讨论和找边界情况的能力。不过 现在才发现现在已经不怎么会写代码了，写出来质量只能说一言难尽，另外还有不少rust语法不熟悉，导致写的很奇怪的东西。 
另外，wordle好玩的！相关文档见https://lab.cs.tsinghua.edu.cn/rust 另外， 清华本门课程质量挺高，感觉小学期写了zwRust两倍以上的代码量。


## 这个函数是干什么的？
在写游戏逻辑的时候，我只有一个原始的想法，就是wordle的6次猜测是彼此相似的，这个过程可以被抽象成一个game_round函数，另一方面，有一个控制整个游戏流程的函数来控制最后获胜/失败提示等。但是这个东西太过程式了，耦合了输入输出，所以明明game_round一模一样却无法在GUI里复用。
正确的做法应该是分离IO和逻辑本身，在一套流程里只留**一个**完全无法复用的过程式性比较强的函数，或者说仔细看gameround，其实里面发生了三件事情：处理判断输入是否合法、改变猜测字母表的状态，输出。这显然是不合理的
```rust
fn game_round(config_info : &parseconfig::MergedConfig , game_info : &GameState, alphabet : &mut HashMap<char,Color>) -> (String,[Color;5]) {

    let mut new_guess = String::new();
    let mut trimmed_guess;
    loop{
        new_guess.clear();
        io::stdin().read_line(&mut new_guess).expect("IO Error");
        trimmed_guess = new_guess.trim_end();
        let checker = match config_info.difficult{
            true => check_valid_guess_difficult,
            false => check_valid_guess,
        };
        if checker(trimmed_guess.to_string(),game_info){
            break;
        }
        else{
            println!("INVALID");
        }
    }

    let color_info = check_word(&game_info.word,trimmed_guess);

    for i in 0..5{
        let letter = new_guess.chars().nth(i).expect("UNREACHABLE");
        let color = &color_info[i];
        if *color == Color::GREEN{
            alphabet.insert(letter.clone(),Color::GREEN);
        }
        else if *color == Color::YELLOW && alphabet[&letter] != Color::GREEN{
            alphabet.insert(letter.clone(),Color::YELLOW);
        }
        else if *color == Color::RED && alphabet[&letter] != Color::GREEN && alphabet[&letter] != Color::YELLOW{
            alphabet.insert(letter.clone(),Color::RED);
        }
    }
    
    if config_info.is_tty{
        for i in 0..5{
            color_info[i].test_print();
        }
        println!("");

        for letter in 'A'..='Z'{
            alphabet[&letter].test_print();
        }
        println!("");
    }
    else {
        for i in 0..5{
            color_info[i].colored_print(new_guess.chars().nth(i).expect("UNREACHABLE"));
        }
        print!(" ");
        for letter in 'A'..='Z'{
            alphabet[&letter].colored_print(letter);
        }
        println!("");
    }

    (trimmed_guess.to_string(),color_info)
}
```
而下面的写法就合理的多，这样和IO相关的问题全部由multi_round处理的，于是gui里就能用了。

```rust
fn game_round(config_info : &MergedConfig , game_info : &mut GameState , guess_arg : String) -> (String,[Color;5]) {
    let color_info = check_word(&game_info.word,&guess_arg);
    paint_keyboad(game_info, color_info, &guess_arg);
    (guess_arg.to_string(),color_info)
}
```


## 签名先写了再说
```rust
fn game_round(config_info : &parseconfig::MergedConfig , game_info : &GameState, alphabet : &mut HashMap<char,Color>)
```
rust的所有权机制使得结构体不能和其成分一起被传入函数里，所以把alphabet放入结构体之后整个函数都得改，不能只改调用的时候，用game_info.alphabet;


## 这是一个FSM
如何表达GUI上三十个格子呢？可以Vec，也可以用[[ Option<char>;5 ] ; 6 ]，也可以用一个Vec。我更习惯变量直接映射客观存在（学物理学的）。但重要的是想明白Enter、BackSpace、输入字母怎么影响状态的改变。首先，每个字符有空、满、锁定三个状态，其中锁定必须按行锁定。那么下面的逻辑正确吗？
- insert: 当存在一个没有被锁定且非满的行的时候可以insert
- backspace: 当存在一个没有被锁定且非空的行的时候可以删除
- enter: 当存在一个满的行且没有被锁定的时候可以enter  
**其实insert存在错误**
- **insert（正确）** :  当**不存在一个没被enter（满且未锁定）的行且**存在一个没有被锁定且非满的行的时候可以insert

```rust
pub struct MyApp {
    pub board_letter : [[Option<char> ; 5 ] ; 6 ],
    pub board_color  : [[Option<Color> ; 5 ] ; 6 ],
...
}
```
于是添加一些额外的上述提到的性质定义在这，locked的更新非常简单，在确定entered的单词有效的时候更新。对于full和emp开始我选择显式定义这些变量在结构体里，回看的时候感觉是很不恰当的，因为这还涉及到更新问题，显然直full/emp其实只是[j][0].is_none()或者[j][4].is_some()的别名。
```rust
{
...
    pub row_lock : [bool ; 6], // only lock is needed
...
}
```

## 解析cmd
Wordle的文档很完整，但是同时很混乱，各种要求分散在各处，比如有如下十个必选参数，斜体不算，下方仅在上方选中之后可选，default表示具有默认值，任何游戏需要这样的参数，而option表示支线功能所需的参数。我写完了发现少些了一个，非常难绷。**阅读理解是重要难点** 另外gui可能还要再扩展一个。特别的，random和word必二选一（clap内置）。
|  *default* | *option*  | word | random |
|----------|---------|------|--------|
| final-set| state    | -    | seed   |
| acceptable-set| config|-  | day    |
| difficult| - | - | -|stats

还有一个要想清楚的事情是在合并JSON和cmd命令的时候彼此可能的冲突情况，大致有几类，直接复制注释了
- seed/day/stats canbe set by JSON or cmd ONLY IN RANDOM MODE, cmd args have higher priority -> 在非random下设置直接报错
- 任何情况下JSON尝试设置word都是不可能的，因为cmd必须给出word（覆盖）或者random（与word互斥）
- final_set/acc_set/state_path/difficult canbe set by JSON or cmd IN ANY MODE, cmd args have higher priority
- config 不再使用，被丢弃


## 亿些杂项

### 所以我的头文件呢？
Rust一大好处是没有声明、定义之类的麻烦，只要写了编译器就能找到。于是很自然地就没有头文件了。但必然有某些数据结构在所有模块里都被提到，于是还得创造出一个类似头文件的东西，把可能造成循环依赖的部分放进去。  

### BUFFED-IO
Rust继承了C系的输入输出缓冲区和没事不刷新缓冲区的习惯，所以print!()的内容可能在任何你意想不到的地方出现。  
~~所以~~
```C
printf("A");
if (fork() && fork()){
    printf("A");
}
```
~~到底输出几个~~

### 少套点Option吧
通常来讲，Vec无需在外面套Option，什么都Option下场就是自己看不懂（

### egui
很好用的ui库，让我一点并发的语法不会顺利写完了UI（，他绘制画布的主要函数是阻塞、循环的，~~arduinoloop~~.所以所有游戏逻辑拆分调用接口的粒度要和UI每次改变的粒度相同，可以写游戏逻辑的时候就想着。~~然而还是不会并发，下个lab或者什么时候再见吧，燃尽了懒得写oj了~~

### 善于default trait、UI排版之类的差不多到了，对不齐就随便调调参数，不行就换一种方法

### 也没有反射、没有继承

## 为什么是语言神？
```txt
      _.-^^---....,,--       
  _--'        _   _  `-._    
 <            ( ) ( )    >   
 |   .-"""-.   \_/      |    
  \ /  .-.  \  / \     /     
   :  (   )  :/   \   /      
    \  `-'  /       '-'      
     `-----'   这是chatGPT画的 Ferris 🦀
```
- 虽然不是那么大众的语言（某同学瑞平只有炒币的用？）但是现在ai支持也很好了，GPT生成的代码大部分都能跑通，~~生成的东西比Coq和Why3好多了~~
- cargo确实好用，难以想象在C++项目中引入三个以上依赖会是什么场面。之前想同时用qt和eigen就很费劲了~~最后摆了~~
- 表达式返回值的机制非常好用，随时写一个{}就是一个完整的lambda表达式，还捕获了外面的变量，直接写一个不带分号的就是返回值，比如

```rust
let new_record = JSONAux{
            answer : Some(new_game_state.word.clone()),
            guesses : {
                let mut temp = Vec::new();
                for item in &new_game_state.trys{
                    // print!("{}",item.0);
                    temp.push(item.0.clone());
                }
                Some(temp)
            }
        };
```
- 编译器建议非常准确（虽然有的我看不懂
- 解决了C很多难绷的细节问题，比如 if a = b