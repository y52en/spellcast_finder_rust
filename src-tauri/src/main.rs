// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering, AtomicU8};
use std::sync::mpsc::{channel, Sender};
use std::time::Instant;

use serde::{Deserialize, Serialize};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn exec(input: &str, swap: u8) -> String {
    CAN_USE_SWAP.store(swap, Ordering::Relaxed);
    let start = Instant::now();

    let (sender, receiver) = channel::<(History, HistoryChar)>();
    let (sender_swap, receiver_swap) = channel::<(History, String, String, u8)>();
    let word_dic: THashTable = serde_json::from_str(include_str!("./word_tree.json")).unwrap();

    let map = input;
    let tbl: Table = parse(&map);

    for x in 0..TABLE_SIZE {
        for y in 0..TABLE_SIZE {
            let tbl = tbl.clone();
            let word_dic = word_dic.clone();
            let sender_swap = sender_swap.clone();
            let sender = sender.clone();
            let mut history_ = History::new();
            let mut history_char = HistoryChar::new();

            std::thread::spawn(move || {
                let key = tbl[x][y].char_cell.get_char();
                let mut result_string = key.to_string();

                search(
                    (x, y),
                    &mut history_,
                    &mut history_char,
                    tbl,
                    &word_dic.inner[&key],
                    &sender,
                    &sender_swap,
                    get_can_use_swap(),
                    &mut result_string,
                );

                if get_can_use_swap() >= 1 {
                    for (chr, t_hash) in &word_dic.inner {
                        if key == *chr {
                            continue;
                        }
                        let mut result_string = chr.to_string();
                        search(
                            (x, y),
                            &mut history_,
                            &mut history_char,
                            tbl,
                            t_hash,
                            &sender,
                            &sender_swap,
                            get_can_use_swap() - 1,
                            &mut result_string,
                        );
                    }
                }

                let mut history_ = History::new();
                history_.length = u8::MAX;
                sender.send((history_, history_char)).unwrap();
            });
        }
    }

    let mut remain = TABLE_SIZE * TABLE_SIZE;
    let mut max_score = 0;
    let mut max_score_swap = [0; 3 as usize];

    let mut max_score_bak: (History, HistoryChar) = (History::new(), HistoryChar::new());
    // History, String, String_SwapBefore
    let mut max_score_swap_bak: Vec<(History, String, String)> =
        vec![(History::new(), String::new(), String::new()); 3 as usize];

    let mut end_flag = false;
    loop {
        let r = receiver.try_recv();
        let r_swap = receiver_swap.try_recv();
        if r.is_err() && r_swap.is_err() {
            if end_flag {
                break;
            }
            println!("no data");
            sleep(100);
            continue;
        }
        if r_swap.is_ok() {
            let (mut history, path_from_string, solve_string, swap_use_time) = r_swap.unwrap();
            let score = calc_score(&mut history, tbl, &solve_string.clone());
            let swap_use_time = swap_use_time as usize;
            if max_score_swap[swap_use_time - 1] < score
                || (max_score_swap[swap_use_time - 1] == score
                    && max_score_swap_bak[swap_use_time - 1].1.len() < solve_string.len())
            {
                max_score_swap[swap_use_time - 1] = score;
                println!(
                    "{}",
                    paths_to_string(
                        &mut history,
                        Some((path_from_string.clone(), solve_string.clone())),
                    )
                );
                println!("swap word: {}", &solve_string);
                println!("swap max_score: {}", score);

                max_score_swap_bak[swap_use_time - 1] = (history, path_from_string, solve_string);
            }
        }
        if r.is_ok() {
            let (mut history, history_char) = r.unwrap();
            if history.length == u8::MAX {
                remain -= 1;
                if remain == 0 {
                    end_flag = true;
                }
                continue;
            }
            let score = calc_score(&mut history, tbl, &history_char.to_string());
            if max_score < score {
                max_score = score;
                println!("{}", paths_to_string(&mut history, None));
                println!("word: {}", history_char.to_string());
                println!("max_score: {}", max_score);

                max_score_bak = (history, history_char);
            }
        }
    }

    let mut out = String::new();
    macro_rules! println_override {
        ($fmt:expr) => {
            println!($fmt);
            out.push_str($fmt);
            out.push_str("\n");
        };
        ($fmt:expr, $($arg:tt)*) => {
            println!($fmt, $($arg)*);
            out.push_str(&format!($fmt, $($arg)*));
            out.push_str("\n");
        };
    }
    macro_rules! print_override {
        ($fmt:expr) => {
            print!("{}", $fmt);
            out.push_str($fmt);
        };
        ($fmt:expr, $($arg:tt)*) => {
            print!($fmt, $($arg)*);
            out.push_str(&format!($fmt, $($arg)*));
        };
    }

    let end = start.elapsed();
    println_override!("");
    println_override!("result:--------------------------");
    println_override!("{}", paths_to_string(&mut max_score_bak.0, None));
    print_override!("word: {} ", max_score_bak.1.to_string());
    println_override!("{} {}", "score:", max_score.to_string());

    println_override!("");
    println_override!("result swap:---------------------");
    for swap_use in 0..get_can_use_swap() {
        let swap_use_index = swap_use as usize;
        let max_score_swap_bak = &max_score_swap_bak[swap_use_index];
        let max_score_swap = max_score_swap[swap_use_index];

        let mut tmp_max_score_swap_bak_0 = max_score_swap_bak.0;
        println_override!(
            "{}",
            paths_to_string(
                &mut tmp_max_score_swap_bak_0,
                Some((
                    max_score_swap_bak.1.to_string(),
                    max_score_swap_bak.2.to_string(),
                )),
            )
        );
        print_override!("swap word: {} ", max_score_swap_bak.2.to_string());
        println_override!("{} {}", "score:", max_score_swap.to_string());
        let diff = get_diff_string_index(
            max_score_swap_bak.1.to_string(),
            max_score_swap_bak.2.to_string(),
        );
        print_override!("swap: ");
        for i in diff {
            let before = max_score_swap_bak.1.get(i..(i + 1));
            let after = max_score_swap_bak.2.get(i..(i + 1));
            print_override!("{}->{} ", before.unwrap(), after.unwrap());
        }
        println_override!("");
        println_override!("-----------------------------------");
        println_override!("");
    }

    println_override!("finish");
    println_override!("total: {:?}", end);
    out
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![exec])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

const EMPTY: char = '_';
const WORD_EOF: char = '@';
const HISTORY_SIZE: usize = 25;

static CAN_USE_SWAP: AtomicU8 = AtomicU8::new(3);
static DIAMOND_MODE: AtomicBool = AtomicBool::new(false);
static DIAMOND_POINT: AtomicI32 = AtomicI32::new(1);

fn get_diamond_point() -> i32 {
    DIAMOND_POINT.load(Ordering::Relaxed)
}

fn get_is_diamond_mode() -> bool {
    DIAMOND_MODE.load(Ordering::Relaxed)
}

fn get_can_use_swap() -> u8 {
    CAN_USE_SWAP.load(Ordering::Relaxed)
}

#[rustfmt::skip]
const ALPHABET_SCORES: [i32; 26] = [
 // A, B, C, D, E, F, G,
    1, 4, 5, 3, 1, 5, 3,
 // H, I, J, K, L, M, N,
    4, 1, 7, 6, 3, 4, 2,
 // O, P, Q, R, S, T, U,
    1, 4, 8, 2, 2, 2, 4,
 // V, W, X, Y, Z
    5, 5, 7, 4, 8
];

macro_rules! vec2d_add {
    ($a:expr, $b:expr) => {
        (
            ($a.0 as isize + $b.0 as isize) as _,
            ($a.1 as isize + $b.1 as isize) as _,
        )
    };
}

macro_rules! conv_vec2d {
    ($a:expr) => {
        ($a.0 as usize, $a.1 as usize)
    };
}

fn calc_score(history: &mut History, tbl: Table, string: &str) -> i32 {
    let chars: Vec<char> = string.chars().collect();
    let mut i = 0;
    let mut score = 0;
    let mut double_score = false;
    let diamond_point = get_diamond_point();
    let is_diamond_mode = get_is_diamond_mode();
    let mut diamonds = 0;
    history.for_each(&mut |vec: (u8, u8)| {
        let cell = &tbl[vec.0 as usize][vec.1 as usize];
        if cell.state_cell.get_config(StateConfig::HasDiamonds) {
            diamonds += diamond_point;
        }
        if is_diamond_mode {
            return;
        }
        score += ALPHABET_SCORES[(chars[i] as u8 - b'a') as usize] * cell.state_cell.get_rate();
        if cell.state_cell.get_config(StateConfig::DoubleScore) {
            double_score = true;
        }
        i += 1;
    });
    if double_score {
        score *= 2;
    }
    if chars.len() >= 6 && !is_diamond_mode {
        score += 10;
    }
    score += diamonds;

    score
}

fn parse(input: &str) -> Table {
    let mut output: Table = [[TableCell {
        char_cell: CharCell::new(EMPTY),
        state_cell: StateCell::new(false, false, false, false),
    }; TABLE_SIZE]; TABLE_SIZE];

    let mut previous_vec: Option<(u8, u8)> = None;
    let mut i = 0;
    let mut lines = 0;
    for c in input.chars() {
        let (x, y) = (i / TABLE_SIZE, i % TABLE_SIZE);
        if lines >= TABLE_SIZE {
            if let Some(digit) = c.to_digit(10) {
                DIAMOND_POINT.store(digit as i32, Ordering::Relaxed);
            } else {
                // only diamond
                if c == 'd' {
                    DIAMOND_MODE.store(true, Ordering::Relaxed);
                } else {
                    panic!("Invalid input");
                }
            }
            continue;
        }
        if let Some(digit) = c.to_digit(10) {
            if let Some(pv) = previous_vec {
                let conf = match digit {
                    0 => StateConfig::DoubleScore,
                    1 => StateConfig::HasDiamonds,
                    2 => StateConfig::DoublePoint,
                    3 => StateConfig::TriplePoint,
                    _ => panic!("Invalid input"),
                };
                output[pv.0 as usize][pv.1 as usize]
                    .state_cell
                    .set_config(conf);
            } else {
                panic!("Invalid input");
            }
        } else if c == '\n' {
            // 0 1 2 3 4
            // 5 ..
            if i % TABLE_SIZE != 0 {
                panic!("Invalid input: line {}:{}", x, y);
            }
            lines += 1;
            continue;
        } else {
            output[x][y].char_cell.set_char(c);
            previous_vec = Some((x as u8, y as u8));
            i += 1;
        }
    }

    output
}

fn get_diff_string_index(a: String, b: String) -> Vec<usize> {
    let mut out = vec![];
    let mut i = 0;
    for (ca, cb) in a.chars().zip(b.chars()) {
        if ca != cb {
            out.push(i);
        }
        i += 1;
    }
    return out;
}

fn paths_to_string(history: &mut History, swap_string: Option<(String, String)>) -> String {
    let mut tbl = vec![vec![EMPTY.to_string(); TABLE_SIZE]; TABLE_SIZE];

    let mut swap_indexes = HashSet::new();
    if swap_string.is_some() {
        let (a, b) = swap_string.unwrap();
        for i in get_diff_string_index(a, b) {
            swap_indexes.insert(i);
        }
    }

    let mut i = 1;
    let mut before_vec = (0, 0);
    history.for_each(&mut |vec: (u8, u8)| {
        tbl[vec.0 as usize][vec.1 as usize] = format!("{}", i);
        if swap_indexes.contains(&(i - 1)) {
            tbl[vec.0 as usize][vec.1 as usize] = tbl[vec.0 as usize][vec.1 as usize].to_string();
        }
        i += 1;
        before_vec = vec;
    });

    let mut out = String::new();
    for i in 0..TABLE_SIZE {
        for j in 0..TABLE_SIZE {
            // print!("{} ", tbl[i][j]);
            out.push_str(&format!("{} ", tbl[i][j]));
        }
        // println!("");
        out.push_str("\n");
    }
    out
}

fn search(
    visit_index: Vec2d,
    history_: &mut History,
    history_char: &mut HistoryChar,
    tbl: Table,
    word_tree: &THashTable,
    sender: &Sender<(History, HistoryChar)>,
    sender_swap: &Sender<(History, String, String, u8)>,
    remain_swap: u8,
    result_string: &mut String,
) {
    history_.append(visit_index);
    history_char.append(tbl[visit_index.0][visit_index.1].char_cell.get_char());

    let english_word: String = history_char._history[0..history_char.length as usize]
        .iter()
        .collect::<String>();
    // has word (@を使うのはchar型の都合上)
    if word_tree.inner.contains_key(&WORD_EOF) {
        if remain_swap == get_can_use_swap() {
            sender
                .send((history_.clone(), history_char.clone()))
                .unwrap();
        } else {
            sender_swap
                .send((
                    history_.clone(),
                    english_word.clone(),
                    result_string.clone(),
                    get_can_use_swap() - remain_swap,
                ))
                .unwrap();
        }
    }

    for mov in MOVES {
        let next_index = vec2d_add!(visit_index, mov);
        if overflow_check(next_index).is_err() {
            continue;
        }
        let next_index = conv_vec2d!(next_index);
        if history_.visited(next_index) {
            continue;
        }

        let next_char = tbl[next_index.0][next_index.1].char_cell.get_char();
        let mut result_string_ = result_string.clone();
        result_string_.push(next_char);
        if word_tree.inner.contains_key(&next_char) {
            search(
                next_index,
                history_,
                history_char,
                tbl,
                &word_tree.inner[&next_char],
                sender,
                sender_swap,
                remain_swap,
                &mut result_string_,
            );
        }

        if remain_swap >= 1 {
            for (chr, t_hash) in &word_tree.inner {
                let mut result_string = result_string.clone();
                result_string.push(*chr);
                if next_char == *chr {
                    continue;
                }
                search(
                    next_index,
                    history_,
                    history_char,
                    tbl,
                    &t_hash,
                    sender,
                    sender_swap,
                    remain_swap - 1,
                    &mut result_string,
                );
            }
        }
    }

    history_.remove();
    history_char.remove();
}

#[inline(always)]
fn overflow_check(vec_: (isize, isize)) -> Result<(), ()> {
    let (x, y) = vec_;
    if x < 0 || TABLE_SIZE <= x as usize {
        return Err(());
    }
    if y < 0 || TABLE_SIZE <= y as usize {
        return Err(());
    }
    return Ok(());
}

type Vec2d = (usize, usize);

// char_table
// char  (a-z)    5-bit : 0b00011111
#[derive(Debug, Clone, Copy)]
struct CharCell {
    _state: char,
}

impl CharCell {
    #[inline(always)]
    fn new(chr: char) -> CharCell {
        CharCell { _state: chr }
    }

    #[inline(always)]
    fn get_char(self) -> char {
        // if add field, use mask
        self._state
    }

    #[inline(always)]
    fn set_char(&mut self, chr: char) {
        // if add field, use mask
        self._state = chr;
    }
}

// state
// score (max 32) 5-bit : 0b00011111
// visited        1-bit : 0b00100000
// double_score   1-bit : 0b01000000
// has-diamond    1-bit : 0b10000000
#[derive(Debug, Clone, Copy)]
struct StateCell {
    _state: u8,
}

enum StateConfig {
    HasDiamonds = 0b00010000,
    DoublePoint = 0b00100000,
    DoubleScore = 0b01000000,
    TriplePoint = 0b10000000,
}

impl StateCell {
    #[inline(always)]
    fn new(
        has_diamonds: bool,
        double_point: bool,
        double_score: bool,
        triple_point: bool,
    ) -> StateCell {
        let mut state: u8 = 0;
        if double_point {
            state |= StateConfig::DoublePoint as u8;
        }
        if double_score {
            state |= StateConfig::DoubleScore as u8;
        }
        if triple_point {
            state |= StateConfig::TriplePoint as u8;
        }
        if has_diamonds {
            state |= StateConfig::HasDiamonds as u8;
        }
        StateCell { _state: state }
    }

    fn get_config(self, config: StateConfig) -> bool {
        self._state & config as u8 != 0
    }

    fn set_config(&mut self, config: StateConfig) {
        self._state |= config as u8;
    }

    #[inline(always)]
    fn get_rate(self) -> i32 {
        if self.get_config(StateConfig::DoublePoint) {
            2
        } else if self.get_config(StateConfig::TriplePoint) {
            3
        } else {
            1
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TableCell {
    char_cell: CharCell,
    state_cell: StateCell,
}

#[derive(Debug, Clone, Copy)]
struct History {
    pub length: u8,
    _history: [u8; HISTORY_SIZE],
}

impl History {
    #[inline(always)]
    fn new() -> History {
        History {
            length: 0,
            _history: [0; HISTORY_SIZE],
        }
    }

    #[inline(always)]
    fn append(&mut self, vec2d: Vec2d) {
        let vec = vec2d.0 * 5 + vec2d.1;
        self._history[self.length as usize] = vec as u8;
        self.length += 1;
    }

    #[inline(always)]
    fn remove(&mut self) {
        self.length -= 1;
    }

    #[inline(always)]
    fn for_each<F: FnMut((u8, u8))>(&mut self, mut func: F) {
        for i in 0..self.length {
            let vec_ = (self._history[i as usize] / 5, self._history[i as usize] % 5);
            func(vec_);
        }
    }

    #[inline(always)]
    fn visited(&self, vec: Vec2d) -> bool {
        let vec = vec.0 * 5 + vec.1;
        for i in 0..self.length {
            if self._history[i as usize] == vec as u8 {
                return true;
            }
        }
        return false;
    }
}

#[derive(Debug, Clone, Copy)]
struct HistoryChar {
    length: u8,
    _history: [char; HISTORY_SIZE],
}

impl HistoryChar {
    #[inline(always)]
    fn new() -> HistoryChar {
        HistoryChar {
            length: 0,
            _history: [EMPTY; HISTORY_SIZE],
        }
    }

    #[inline(always)]
    fn append(&mut self, chr: char) {
        self._history[self.length as usize] = chr;
        self.length += 1;
    }

    #[inline(always)]
    fn remove(&mut self) {
        self.length -= 1;
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn for_each(&self, func: fn(char)) {
        for i in 0..self.length {
            func(self._history[i as usize]);
        }
    }

    #[inline(always)]
    fn to_string(&self) -> String {
        let mut s = String::new();
        for i in 0..self.length {
            s.push(self._history[i as usize]);
        }
        s
    }
}

const TABLE_SIZE: usize = 5;
type Table = [[TableCell; TABLE_SIZE]; TABLE_SIZE];

#[rustfmt::skip]
const MOVES: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    ( 0, -1),          ( 0, 1),
    ( 1, -1), ( 1, 0), ( 1, 1),
];

fn sleep(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct THashTable {
    pub inner: HashMap<char, THashTable>,
}
