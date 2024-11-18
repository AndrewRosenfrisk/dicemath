use crossterm::{
    cursor::{Hide, MoveTo},
    execute,
    terminal::{
        Clear,
        ClearType::{All, Purge},
        DisableLineWrap,
    },
};
use rand::Rng;
use std::{
    io::{stdin, stdout},
    sync::mpsc::channel,
    thread,
    time::{Duration, SystemTime},
};

const CANVAS_WIDTH: u16 = 79;
const CANVAS_HEIGHT: u16 = 21;
const DIE_WIDTH: u16 = 9;
const DIE_HEIGHT: u16 = 5;
const QUIZ_TIME: u64 = 30;
const REWARD: u16 = 4;
const PENALTY: u16 = 1;
const D1: [&str; 5] = [
    "+-------+",
    "|       |",
    "|   O   |",
    "|       |",
    "+-------+",
];
const D2A: [&str; 5] = [
    "+-------+",
    "| O     |",
    "|       |",
    "|     O |",
    "+-------+",
];
const D2B: [&str; 5] = [
    "+-------+",
    "|     O |",
    "|       |",
    "| O     |",
    "+-------+",
];
const D3A: [&str; 5] = [
    "+-------+",
    "| O     |",
    "|   O   |",
    "|     O |",
    "+-------+",
];
const D3B: [&str; 5] = [
    "+-------+",
    "|     O |",
    "|   O   |",
    "| O     |",
    "+-------+",
];
const D4: [&str; 5] = [
    "+-------+",
    "| O   O |",
    "|       |",
    "| O   O |",
    "+-------+",
];
const D5: [&str; 5] = [
    "+-------+",
    "| O   O |",
    "|   O   |",
    "| O   O |",
    "+-------+",
];
const D6A: [&str; 5] = [
    "+-------+",
    "| O O O |",
    "|       |",
    "| O O O |",
    "+-------+",
];
const D6B: [&str; 5] = [
    "+-------+",
    "| O   O |",
    "| O   O |",
    "| O   O |",
    "+-------+",
];

fn main() -> Result<(), std::io::Error> {
    println!("Add up the sides of all the dice displayed on the screen. You have {} seconds to answer as many as possible. You get {} points for each correct answer and lose {} point for each incorrect answer.", QUIZ_TIME, REWARD, PENALTY);
    println!("Press Enter to begin...");
    'input: loop {
        let mut input = String::new();
        stdin().read_line(&mut input)?;

        if input.contains("\n") {
            break 'input;
        } else {
            println!("Invalid input. Please try again.");
            continue;
        }
    }
    let mut correct_count = 0;
    let mut incorrect_count = 0;
    let mut rng = rand::thread_rng();

    let start_time = SystemTime::now();

    while start_time.elapsed().unwrap().as_secs() < QUIZ_TIME {
        execute!(stdout(), Hide, Clear(Purge), Clear(All), DisableLineWrap)?;

        //create a list of all possible points where dice will be drawn on the canvas
        let mut valid_points: Vec<Point> = vec![];
        for x in 1..CANVAS_WIDTH - 1 - DIE_WIDTH {
            for y in 1..CANVAS_HEIGHT - 1 - DIE_HEIGHT {
                valid_points.push(Point(x, y));
            }
        }

        let dice_count = rng.gen_range(2..=6);

        let mut dice: Vec<(DICE, Point)> = vec![];

        let mut sum_answer: u32 = 0;

        for _ in 1..=dice_count {
            let die_value: u8 = rng.gen_range(1..=6);
            sum_answer += Into::<u32>::into(die_value);
            let point = valid_points.swap_remove(rng.gen_range(0..valid_points.len()));

            valid_points = filter_invalid_points(&mut valid_points, point);

            dice.push((get_dice_from_number(die_value).unwrap(), point));
        }
        for die in dice {
            die.0.print_die(die.1)?;
        }
        //move outside the canvas to prevent overwriting dice images
        execute!(stdout(), MoveTo(1, 22))?;
        println!("Enter the sum: ");

        //create a second thread to allow for the game timer to interrupt waiting for player input.
        let (tx, rx) = channel::<String>();

        let input_handle = thread::spawn(move || {
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            tx.send(input).unwrap();
        });

        let input = rx
            .recv_timeout(Duration::from_secs(
                QUIZ_TIME - start_time.elapsed().unwrap().as_secs(),
            ))
            .unwrap_or_else(|_| {
                input_handle.thread().unpark();
                "Timeout".to_string()
            });

        if input == "Timeout" {
            //don't penalize not answering the last question in time.
            println!("\nYou ran out of time! The answer was {}", sum_answer);
        } else if input.trim().parse::<u32>().is_ok()
            && sum_answer == input.trim().parse::<u32>().unwrap()
        {
            correct_count += 1;
        } else {
            println!("Incorrect. The answer is {}", sum_answer);
            incorrect_count += 1;
            thread::sleep(Duration::from_secs(2));
        }
    }
    let score: i32 =
        Into::<i32>::into(correct_count * REWARD) - Into::<i32>::into(incorrect_count * PENALTY);
    println!(
        "Correct: {}\nIncorrect: {}\nScore: {}",
        correct_count, incorrect_count, score
    );

    Ok(())
}

enum DICE {
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
}

impl DICE {
    fn get_lines(&self) -> [&str; 5] {
        let mut rng = rand::thread_rng();
        match self {
            DICE::ONE => D1,
            DICE::TWO => {
                if rng.gen_bool(0.5) {
                    D2A
                } else {
                    D2B
                }
            }
            DICE::THREE => {
                if rng.gen_bool(0.5) {
                    D3A
                } else {
                    D3B
                }
            }
            DICE::FOUR => D4,
            DICE::FIVE => D5,
            DICE::SIX => {
                if rng.gen_bool(0.5) {
                    D6A
                } else {
                    D6B
                }
            }
        }
    }
    fn print_die(&self, point: Point) -> Result<(), std::io::Error> {
        let lines = self.get_lines();
        let mut current_line = 0;
        for line in lines {
            execute!(stdout(), MoveTo(point.0, point.1 + current_line))?;
            println!("{}", line);
            current_line += 1;
        }

        Ok(())
    }
}

#[derive(PartialEq, Copy, Clone)]
struct Point(u16, u16);

fn get_dice_from_number(num: u8) -> Option<DICE> {
    match num {
        1 => Some(DICE::ONE),
        2 => Some(DICE::TWO),
        3 => Some(DICE::THREE),
        4 => Some(DICE::FOUR),
        5 => Some(DICE::FIVE),
        6 => Some(DICE::SIX),
        _ => None,
    }
}

fn filter_invalid_points(valid_points: &mut Vec<Point>, point: Point) -> Vec<Point> {
    //points are invalid if they would cause two dice or more dice to overlap.
    //remove other points within die width/height range of the current point
    //return remaining points

    let lower_x_bound = if point.0 < DIE_WIDTH {
        0
    } else {
        point.0 - DIE_WIDTH
    };
    let lower_y_bound = if point.1 < DIE_HEIGHT {
        0
    } else {
        point.1 - DIE_HEIGHT
    };

    for x in lower_x_bound..point.0 + DIE_WIDTH {
        for y in lower_y_bound..point.1 + DIE_HEIGHT {
            let invalid_point = valid_points
                .clone()
                .into_iter()
                .position(|p| p == Point(x, y));

            if invalid_point.is_some() {
                valid_points.remove(invalid_point.unwrap());
            }
        }
    }

    valid_points.to_vec()
}
