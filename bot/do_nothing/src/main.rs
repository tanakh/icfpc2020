#![allow(non_snake_case, unused, non_upper_case_globals)]
// Copyright 2020 Google LLC
// Copyright 2020 Team Spacecat
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


extern crate rust_game_base;
#[macro_use]
extern crate itertools;

/*
仕様があまり定まっていないので，generic な感じの AI を書く．

- 攻撃側であることを仮定．
- 分裂はしないことにする (自機はつねにひとつ．)．
- 相手は次の段階では動かないことを仮定．

- 上下左右にだけ動く．速度は 1 か 0 のみ．
*/

static mut DEV: bool = false;

fn is_dev() -> bool {
    unsafe { DEV }
}

fn main() {
    if std::env::args().nth(1) == Some("dev".to_owned()) {
        unsafe { DEV = true }
    }
    eprintln!("send_join_request");
    let resp = rust_game_base::send_join_request();

    eprintln!("send_start_request");
    let mut res = rust_game_base::send_start_request(&rust_game_base::Param { energy: 4, laser_power: 4, cool_down_per_turn: 4, life: 4 }).unwrap();

    eprintln!("send_command_request");
    loop {
        if res.current_game_state == rust_game_base::CurrentGameState::END {
            break;
        }
        res = rust_game_base::send_command_request(&mut vec![].into_iter()).unwrap();
    }
}

struct StartParam {}

fn run(mut state: State) {
    loop {
        let mut best = (-10000.0, vec![], None);
        for a in possible_actions(&state) {
            let ns = next_state(&state, &a);
            let val = evaluate_state(&ns);
            if best.0 < val {
                best = (val, a, Some(ns));
            }
        }
        // let got_state = Proxy::do_action(&best.1);

        // TODO: log if state != got_state.

        state = best.2.unwrap();
        if state.finished() {
            return;
        }
    }
}

#[derive(Clone, Debug, Copy)]
struct P {
    x: isize,
    y: isize,
}

impl std::ops::Add for P {
    type Output = P;
    fn add(self, p: Self) -> Self::Output {
        P::new(self.x + p.x, self.y + p.y)
    }
}

impl std::ops::Sub for P {
    type Output = P;
    fn sub(self, p: Self) -> Self::Output {
        P::new(self.x - p.x, self.y - p.y)
    }
}

impl P {
    fn new(x: isize, y: isize) -> P {
        P { x, y }
    }
    fn dist(&self, p: P) -> f64 {
        self.dist2(p).sqrt()
    }
    fn dist2(&self, p: P) -> f64 {
        (*self - p).norm2()
    }
    fn norm2(&self) -> f64 {
        let (x, y) = (self.x, self.y);
        (x * x + y * y) as _
    }
    fn norm(&self) -> f64 {
        self.norm2().sqrt()
    }
}

#[derive(Clone, Debug)]
struct Machine {
    pos: P,
    v: P,
}

#[derive(Clone, Debug)]
struct State {
    me: Machine,
    you: Machine,
    turn: usize,
}

impl State {
    fn from_response(gs: rust_game_base::Response) -> State {
        todo!();
    }

    fn dummy() -> State {
        State {
            turn: 100,
            me: Machine {
                pos: P::new(1, 1),
                v: P::new(0, 0),
            },
            you: Machine {
                pos: P::new(-1, -1),
                v: P::new(0, 0),
            },
        }
    }

    fn finished(&self) -> bool {
        // TODO
        self.turn == 0
    }
}

// the higher the better
fn evaluate_state(s: &State) -> f64 {
    const small: f64 = 1.0 / 1000.0;
    1.0 / (s.me.pos.dist(s.you.pos) + 1.0) - small * s.me.v.norm()
}

// get next states without actually running the action.
fn next_state(s: &State, a: &Action) -> State {
    let mut s = s.clone();
    s.turn -= 1;

    for c in a.iter() {
        match c {
            Command::Thrust(a) => {
                s.me.v = s.me.v + *a;
                s.me.pos = s.me.pos + s.me.v;
            }
            _ => unimplemented!(),
        }
    }
    s
}

type Action = Vec<Command>;

#[derive(Debug, Clone)]
enum Command {
    // 自機の速度を変える。指定したそのターンから即時効果を発揮する。
    // e.g.: Pos(0 . 0), V(1 . 0) の時に Thruster を A(1 . 0) 吹くと、次のターンでは Pos(2 . 0), V(2 . 0)
    Thrust(P), // (dx, dy)
    Bomb,
    Beam { dir: P, power: usize },
}

fn possible_actions(s: &State) -> Vec<Action> {
    let mut res = vec![];

    res.push(vec![]); // do nothing
    for (dx, dy) in iproduct!(-1..=1, -1..=1) {
        if dx == 0 && dy == 0 {
            continue;
        }
        if (dx != 0) && (dy != 0) {
            continue;
        }
        res.push(vec![Command::Thrust(P::new(dx, dy))]);
    }
    res
}

#[cfg(tests)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        assert_eq!(1 + 1, 2);
    }
}
